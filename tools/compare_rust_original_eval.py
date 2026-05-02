#!/usr/bin/env python3
"""Timed Rust `recognize-page` vs NDLOCR-Lite `ocr.py`, then CER/WER (baseline: original).

Requires a clone of ndlocr-lite with `src/ocr.py` (see `--ndlocr-src`).
"""

from __future__ import annotations

import argparse
import shutil
import subprocess
import sys
import time
from pathlib import Path
from typing import List, Tuple

_TOOLS_DIR = Path(__file__).resolve().parent
if str(_TOOLS_DIR) not in sys.path:
    sys.path.insert(0, str(_TOOLS_DIR))

import eval_ocr_metrics as eom  # noqa: E402
import report_rust_ocr_eval as rre  # noqa: E402


def run_original_ocr_timed(
    *,
    python_bin: str,
    ndlocr_src: Path,
    image: Path,
    output_dir: Path,
) -> float:
    output_dir.mkdir(parents=True, exist_ok=True)
    cmd = [
        python_bin,
        "ocr.py",
        "--sourceimg",
        str(image.resolve()),
        "--output",
        str(output_dir.resolve()),
    ]
    t0 = time.perf_counter()
    try:
        subprocess.run(
            cmd,
            cwd=str(ndlocr_src.resolve()),
            check=True,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.PIPE,
            text=True,
        )
    except subprocess.CalledProcessError as e:
        raise SystemExit(f"ocr.py failed for {image}:\n{e.stderr}") from e
    return time.perf_counter() - t0


def resolve_python_bin(raw: str, repo_root: Path) -> str:
    p = Path(raw).expanduser()
    if p.is_absolute() or len(p.parts) > 1:
        # Do not resolve symlinks here: venv `bin/python` is often a symlink to
        # the framework Python, and resolving it would lose the venv site-packages.
        return str(p if p.is_absolute() else (repo_root / p).absolute())
    found = shutil.which(raw)
    return found or raw


def build_compare_report_md(
    *,
    normalize: str,
    truth_count: int,
    rust_timing: List[Tuple[str, float]],
    orig_timing: List[Tuple[str, float]],
    ran_rust: bool,
    ran_orig: bool,
    systems: List[eom.SystemMetrics],
) -> str:
    header = (
        "\n".join(
            [
                "# Rust vs NDLOCR-Lite 比較（速度・精度）",
                "",
                f"- 正規化: `{normalize}`",
                f"- 正解ファイル数: {truth_count}",
                "",
                "- 精度の **baseline（Δ の基準）は NDLOCR-Lite (`original`)** です。",
                "",
            ]
        )
        + "\n"
    )
    rust_sec = rre.build_timing_markdown(
        rust_timing,
        ran_inference=ran_rust and bool(rust_timing),
        section_heading="## Rust（`ndlocr-lite-rs` / `recognize-page`）",
        idle_note_engine="recognize-page",
    )
    orig_sec = rre.build_timing_markdown(
        orig_timing,
        ran_inference=ran_orig and bool(orig_timing),
        section_heading="## NDLOCR-Lite（`ocr.py`）",
        idle_note_engine="ocr.py",
    )
    acc = eom.build_markdown_report(
        systems,
        normalize,
        truth_count,
        baseline_system="original",
    )
    acc_body = acc.replace("# OCR Evaluation Report\n\n", "", 1)
    marker = "## Aggregate Metrics"
    if marker in acc_body:
        acc_body = acc_body[acc_body.index(marker) :]
    return header + rust_sec + orig_sec + acc_body


def parse_args() -> argparse.Namespace:
    root = rre.default_repo_root()
    p = argparse.ArgumentParser(description="Compare timed Rust vs NDLOCR-Lite ocr.py on eval images.")
    p.add_argument("--repo-root", type=Path, default=root)
    p.add_argument("--input-dir", type=Path, required=True)
    p.add_argument("--truth-dir", type=Path, required=True)
    p.add_argument("--output-dir", type=Path, required=True)
    p.add_argument(
        "--ndlocr-src",
        type=Path,
        required=True,
        help="Directory containing ocr.py (e.g. ndlocr-lite clone .../src)",
    )
    p.add_argument("--python", default="python3", help="Python for ocr.py")
    p.add_argument("--normalize", choices=["none", "basic", "strict"], default="strict")
    p.add_argument("--require-all", action="store_true")
    p.add_argument("--no-build", action="store_true")
    p.add_argument("--charset", type=Path, default=root / "ndlocr" / "src" / "config" / "NDLmoji.yaml")
    p.add_argument(
        "--model",
        type=Path,
        default=root
        / "models"
        / "parseq-ndl-24x768-100-tiny-153epoch-tegaki3-r8data-202604.onnx",
    )
    p.add_argument("--binarize-threshold", type=int, default=220)
    p.add_argument("--rule-pack", type=Path, default=None)
    p.add_argument("--line-crop-padding", type=int, default=None)
    p.add_argument("--ort-lib-dir", type=Path, default=None)
    p.add_argument("--binary", type=Path, default=None)
    p.add_argument("--skip-rust", action="store_true")
    p.add_argument("--skip-original", action="store_true")
    return p.parse_args()


def main() -> int:
    args = parse_args()
    repo_root = args.repo_root.expanduser().resolve()
    python_bin = resolve_python_bin(args.python, repo_root)
    ndlocr_src = args.ndlocr_src.expanduser().resolve()
    if not (ndlocr_src / "ocr.py").is_file() and not args.skip_original:
        raise SystemExit(f"ocr.py not found under {ndlocr_src}")
    input_dir = args.input_dir.expanduser().resolve()
    truth_dir = args.truth_dir.expanduser().resolve()
    out = args.output_dir.expanduser().resolve()
    rust_dir = out / "rust"
    orig_dir = out / "original"
    out.mkdir(parents=True, exist_ok=True)
    if not args.skip_rust:
        rust_dir.mkdir(parents=True, exist_ok=True)
    if not args.skip_original:
        orig_dir.mkdir(parents=True, exist_ok=True)

    images = rre.discover_eval_images(input_dir)
    rust_times: List[Tuple[str, float]] = []
    orig_times: List[Tuple[str, float]] = []

    ran_rust = not args.skip_rust
    ran_orig = not args.skip_original

    if ran_rust:
        rp_bin = (
            args.binary.expanduser().resolve()
            if args.binary
            else rre.ensure_release_binary(repo_root, no_build=args.no_build)
        )
        for image in images:
            stem = image.stem
            pred = rust_dir / f"{stem}.txt"
            print(f"[rust] {image.name}", file=sys.stderr)
            sec = rre.run_recognize_page_timed(
                bin_path=rp_bin,
                repo_root=repo_root,
                image=image,
                out_txt=pred,
                model=args.model.expanduser().resolve(),
                charset=args.charset.expanduser().resolve(),
                binarize_threshold=args.binarize_threshold,
                rule_pack=args.rule_pack.expanduser().resolve() if args.rule_pack else None,
                line_crop_padding=args.line_crop_padding,
                ort_lib_dir=args.ort_lib_dir.expanduser().resolve() if args.ort_lib_dir else None,
            )
            rust_times.append((stem, sec))
        rre.write_timing_csv(out / "timing_rust.csv", rust_times)

    if ran_orig:
        for image in images:
            stem = image.stem
            print(f"[original] {image.name}", file=sys.stderr)
            sec = run_original_ocr_timed(
                python_bin=python_bin,
                ndlocr_src=ndlocr_src,
                image=image,
                output_dir=orig_dir,
            )
            orig_times.append((stem, sec))
        rre.write_timing_csv(out / "timing_original.csv", orig_times)

    truth_files = eom.discover_truth_files(truth_dir)
    if not truth_files:
        raise SystemExit(f"no truth *.txt under {truth_dir}")

    systems: List[eom.SystemMetrics] = []
    if not args.skip_rust:
        systems.append(eom.evaluate_system("rust", rust_dir, truth_dir, truth_files, args.normalize))
    if not args.skip_original:
        systems.append(eom.evaluate_system("original", orig_dir, truth_dir, truth_files, args.normalize))

    if len(systems) < 2:
        raise SystemExit("need both rust and original predictions (drop --skip-rust / --skip-original)")

    baseline = "original"
    if args.require_all:
        for s in systems:
            if s.missing_count:
                print(f"error: {s.name!r} missing {s.missing_count}", file=sys.stderr)
                return 2

    eom.write_csv(out / "metrics.csv", systems)
    (out / "metrics.md").write_text(
        eom.build_markdown_report(systems, args.normalize, len(truth_files), baseline_system=baseline),
        encoding="utf-8",
    )
    (out / "report.md").write_text(
        build_compare_report_md(
            normalize=args.normalize,
            truth_count=len(truth_files),
            rust_timing=rust_times,
            orig_timing=orig_times,
            ran_rust=ran_rust,
            ran_orig=ran_orig,
            systems=systems,
        ),
        encoding="utf-8",
    )
    print(f"wrote {out / 'report.md'}", file=sys.stderr)

    ob = next(s for s in systems if s.name == baseline)
    print("system,matched_files,missing_files,cer,wer,delta_cer_vs_original,delta_wer_vs_original")
    for s in sorted(systems, key=lambda x: x.cer):
        print(
            f"{s.name},{s.matched_count},{s.missing_count},{s.cer:.6f},{s.wer:.6f},"
            f"{(s.cer - ob.cer):+.6f},{(s.wer - ob.wer):+.6f}"
        )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
