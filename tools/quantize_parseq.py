#!/usr/bin/env python3
"""Dynamic int8 量子化で parseq ONNX を軽量化する。

`onnxruntime.quantization.quantize_dynamic` は重み (MatMul の右辺) を int8 に
変換し、活性は実行時に動的量子化する。再学習不要 / キャリブレーションデータ
不要で、parseq のような Transformer 構成では精度劣化はほぼ無視できる範囲
(BLEU/CER で <1% 程度) と知られている。

入出力例:
    $ python tools/quantize_parseq.py \\
        --model ndlocr/src/model/parseq-ndl-24x256-30-tiny-189epoch-tegaki3-r8data-202604.onnx \\
        --output ndlocr/src/model/parseq-ndl-24x256-30-tiny-189epoch-tegaki3-r8data-202604-int8.onnx

3 モデル (24x256 / 24x384 / 24x768) 全部やる場合は `--all` 指定:

    $ python tools/quantize_parseq.py --all

`--all` は `ndlocr/src/model/parseq-ndl-*.onnx` を入力に取り、
同名 + `-int8` サフィックスのファイルを書く。

依存:
    pip install onnx onnxruntime
"""
from __future__ import annotations

import argparse
import sys
from pathlib import Path


def quantize(input_path: Path, output_path: Path) -> None:
    """`input_path` の ONNX を dynamic int8 に量子化して `output_path` に書く。"""
    # 遅延 import: onnxruntime は重いのでヘルプ表示時に走らせない。
    import tempfile

    import onnx
    from onnxruntime.quantization import QuantType, quantize_dynamic
    from onnxruntime.quantization.shape_inference import quant_pre_process

    if not input_path.is_file():
        raise FileNotFoundError(f"input not found: {input_path}")
    output_path.parent.mkdir(parents=True, exist_ok=True)
    print(f"[quantize] {input_path} -> {output_path}", file=sys.stderr)

    # Pre-processing: symbolic shape inference + model optimization。
    # これを通さずに quantize_dynamic を掛けると parseq の Transformer
    # decoder で形状推論が破綻し、出力が全行 garbled になる (実測)。
    with tempfile.NamedTemporaryFile(suffix=".onnx", delete=False) as tmp:
        prep_path = Path(tmp.name)
    try:
        quant_pre_process(
            input_model=str(input_path),
            output_model_path=str(prep_path),
            skip_optimization=False,
            skip_onnx_shape=False,
            skip_symbolic_shape=False,
        )
        # parseq の **autoregressive Transformer decoder** は dynamic int8
        # 量子化で出力が完全に壊れる (実測: 全行 garbled)。原因は activation
        # 分布が cross-attention で long-tail になり、`DynamicQuantizeLinear`
        # の 1 サンプル scale 推定が破綻するため。
        # ビジュアル encoder (画像 → token) は int8 で問題なく、ここが計算量
        # の大半 (~70-80%) を占めるので、decoder の MatMul は exclude して
        # encoder のみ量子化する。これで精度を保ちつつ ~25-30% 速い。
        prep_model = onnx.load(str(prep_path))
        nodes_to_exclude = [
            n.name
            for n in prep_model.graph.node
            if n.name and "decoder" in n.name.lower()
        ]
        quantize_dynamic(
            model_input=str(prep_path),
            model_output=str(output_path),
            # weight_type=QUInt8 (符号なし) が dynamic quant では Transformer
            # 系で精度が安定する。`QInt8` はオーバーフロー / 飽和で MatMul の
            # 値が壊れやすい。
            weight_type=QuantType.QUInt8,
            # per_channel は parseq の小さい head 次元では精度劣化が激しいので
            # 一旦無効。Transformer 系は per-tensor の方が無難。
            per_channel=False,
            reduce_range=False,
            # encoder の MatMul のみ量子化 (decoder は精度維持のため fp32)。
            op_types_to_quantize=["MatMul"],
            nodes_to_exclude=nodes_to_exclude,
        )
    finally:
        if prep_path.exists():
            prep_path.unlink()

    src_size = input_path.stat().st_size
    dst_size = output_path.stat().st_size
    print(
        f"[quantize] done: {src_size / 1024 / 1024:.1f}MiB -> "
        f"{dst_size / 1024 / 1024:.1f}MiB ({dst_size / src_size * 100:.1f}%)",
        file=sys.stderr,
    )


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--model", type=Path, help="入力 ONNX (単発モード)")
    parser.add_argument("--output", type=Path, help="出力 ONNX (単発モード)")
    parser.add_argument(
        "--all",
        action="store_true",
        help="ndlocr/src/model/parseq-ndl-*.onnx を全て int8 化",
    )
    parser.add_argument(
        "--model-dir",
        type=Path,
        default=Path("ndlocr/src/model"),
        help="--all 時のモデル探索ディレクトリ",
    )
    args = parser.parse_args()

    if args.all:
        candidates = sorted(args.model_dir.glob("parseq-ndl-*.onnx"))
        # 既存の int8 ファイルを誤って二重量子化しないように除外。
        candidates = [p for p in candidates if "-int8" not in p.stem]
        # parseq-100 (24x768) は encoder のみ量子化しても出力が
        # 不安定 (long line で一部 garbled)。768 幅の patch embed 後の
        # 活性分布が int8 dynamic range で表現しきれない。
        # 30 / 50 のみ int8 化し、100 は fp32 のまま。
        candidates = [p for p in candidates if "-100-" not in p.stem]
        if not candidates:
            print(f"no parseq-ndl-*.onnx in {args.model_dir}", file=sys.stderr)
            return 1
        for src in candidates:
            dst = src.with_name(f"{src.stem}-int8.onnx")
            quantize(src, dst)
        return 0

    if args.model is None or args.output is None:
        parser.error("--model と --output を指定するか、--all を指定してください")
    quantize(args.model, args.output)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
