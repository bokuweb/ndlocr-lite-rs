# Model Build Guide (Local ONNX)

このリポジトリは OCR 実行時にローカル ONNX を参照します。  
モデルは配布物に同梱されていないため、**学習済みチェックポイントから ONNX を生成**して `models/` に配置します。

## どこからモデルを持ってくるか

- DEIMv2: 自前学習チェックポイント（`last.pth` など）
- PARSeq(30/50/100): 自前学習チェックポイント（`.ckpt`）

ベースとなる学習/変換手順は `ndlocr/train/README.md` に準拠します。

学習せず公式モデルを使う場合は、レビュー済みコミット・SHA-256で固定した取得経路を使います。
文字集合とライセンス通知も同じ版から取得します。

```bash
python3 tools/fetch_upstream_assets.py models
python3 tools/fetch_upstream_assets.py models --offline
```

追従先と次回更新の手順は [互換性監査](ocr_pipeline_compatibility.md) を参照してください。

## 事前準備

- DEIMv2 リポジトリを用意し、`ndlocr/train/deimv2code` の内容を適用済み
- PARSeq リポジトリを用意し、`ndlocr/train/parseqcode` の内容を適用済み
- Python 環境に `torch`, `onnx`, `onnxruntime`, `strhub`, `pyyaml` 等が導入済み

## ONNX 生成（この repo から実行）

```bash
chmod +x tools/build_local_models.sh

tools/build_local_models.sh \
  --deim-repo /path/to/DEIMv2 \
  --deim-config /path/to/DEIMv2/configs/ndl_deimv2/deimv2_dinov3_s_coco_r4_800.yml \
  --deim-ckpt /path/to/DEIMv2/outputs/deimv2_dinov3_s_coco_r4_800/last.pth \
  --parseq-ckpt30 /path/to/parseq30.ckpt \
  --parseq-ckpt50 /path/to/parseq50.ckpt \
  --parseq-ckpt100 /path/to/parseq100.ckpt
```

生成されるファイル:

- `models/deim-s-1024x1024.onnx`
- `models/parseq-ndl-24x256-30-tiny-189epoch-tegaki3-r8data-202604.onnx`
- `models/parseq-ndl-24x384-50-tiny-300epoch-tegaki3-r8data-202604.onnx`
- `models/parseq-ndl-24x768-100-tiny-153epoch-tegaki3-r8data-202604.onnx`

## OCR 実行

```bash
cargo run --features onnx -- detect --image /path/to/image.jpg
cargo run --features onnx -- recognize --image /path/to/image.jpg
```
