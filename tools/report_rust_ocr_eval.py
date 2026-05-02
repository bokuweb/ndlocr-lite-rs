#!/usr/bin/env python3
"""Run timed recognize-page, then emit CER/WER + wall-clock Markdown report.

See docs/evaluation_report.md.
"""

from __future__ import annotations

import argparse
import csv
import os
import subprocess
import sys
import time
from pathlib import Path
from typing import List, Sequence, Tuple

_TOOLS_DIR = Path(__file__).resolve().parent
if str(_TOOLS_DIR) not in sys.path:
    sys.path.insert(0, str(_TOOLS_DIR))

import eval_ocr_metrics as eom  # noqa: E402


def discover_eval_images(input_dir: Path) -> List[Path]:
    exts = {".png", ".jpg", ".jpeg", ".ppm"}
    out = sorted(p for p in input_dir.iterdir() if p.suffix.lower() in exts and p.is_file())
    if not out:
        raise SystemExit(f"no images (*.png, *.jpg, *.jpeg, *.ppm) in {input_dir}")
    return out


def default_repo_root() -> Path:
    return Path(__file__).resolve().parent.parent


def release_binary_path(repo_root: Path) -> Path:
    target_dir = os.environ.get("CARGO_TARGET_DIR")
    if target_dir:
        return Path(target_dir).expanduser().resolve() / "release" / "ndlocr-lite-rs"
    return repo_root / "target" / "release" / "ndlocr-lite-rs"


def ensure_release_binary(repo_root: Path, *, no_build: bool) -> Path:
    bin_path = release_binary_path(repo_root)
    if bin_path.is_file():
        return bin_path
    if no_build:
        raise SystemExit(f"release binary not found: {bin_path} (drop --no-build to build)")
    subprocess.run(
        ["cargo", "build", "--release", "--features", "onnx"],
        cwd=repo_root,
        check=True,
    )
    if not bin_path.is_file():
        raise SystemExit(f"build finished but binary missing: {bin_path}")
    return bin_path


def write_timing_csv(path: Path, rows: Sequence[Tuple[str, float]]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("w", encoding="utf-8", newline="") as f:
        w = csv.writer(f)
        w.writerow(["stem", "wall_seconds"])
        for stem, sec in rows:
            w.writerow([stem, f"{sec:.6f}"])


def build_timing_markdown(
    rows: Sequence[Tuple[str, float]],
    *,
    ran_inference: bool,
    section_heading: str = "## 処理時間 (wall-clock)",
    idle_note_engine: str = "recognize-page",
) -> str:
    lines: List[str] = []
    lines.append(section_heading)
    lines.append("")
    if not ran_inference or not rows:
        lines.append(f"（この実行では `{idle_note_engine}` の計測を行っていません）")
        lines.append("")
        return "\n".join(lines)
    total = sum(sec for _, sec in rows)
    mean = total / len(rows)
    lines.append(f"- 合計: **{total:.3f} s**（{len(rows)} 画像）")
    lines.append(f"- 平均: **{mean:.3f} s / 画像**")
    lines.append("")
    lines.append("| stem | seconds |")
    lines.append("|---:|---:|")
    for stem, sec in rows:
        lines.append(f"| {stem} | {sec:.6f} |")
    lines.append("")
    return "\n".join(lines) + "\n"


def build_combined_report_md(
    *,
    normalize: str,
    truth_count: int,
    timing_rows: Sequence[Tuple[str, float]],
    systems: List[eom.SystemMetrics],
    baseline_system: str | None,
    ran_inference: bool,
) -> str:
    header = "\n".join(
        [
            "# Rust OCR 評価: 速度・精度",
            "",
            f"- 正規化: `{normalize}`",
            f"- 正解ファイル数（真値ツリーより）: {truth_count}",
            "",
        ]
    ) + "\n"
    timing = build_timing_markdown(timing_rows, ran_inference=ran_inference)
    acc = eom.build_markdown_report(
        systems,
        normalize,
        truth_count,
        baseline_system=baseline_system,
    )
    acc_body = acc.replace("# OCR Evaluation Report\n\n", "", 1)
    marker = "## Aggregate Metrics"
    if marker in acc_body:
        acc_body = acc_body[acc_body.index(marker) :]
    return header + timing + acc_body


def run_recognize_page_timed(
    *,
    bin_path: Path,
    repo_root: Path,
    image: Path,
    out_txt: Path,
    model: Path,
    charset: Path,
    binarize_threshold: int,
    rule_pack: Path | None,
    line_crop_padding: int | None,
    ort_lib_dir: Path | None,
) -> float:
    cmd: List[str] = [
        str(bin_path),
        "recognize-page",
        "--image",
        str(image.resolve()),
        "--model",
        str(model),
        "--charset",
        str(charset),
        "--binarize-threshold",
        str(binarize_threshold),
        "--output-txt",
        str(out_txt.resolve()),
    ]
    if rule_pack is not None:
        cmd += ["--rule-pack", str(rule_pack)]
    if line_crop_padding is not None:
        cmd += ["--line-crop-padding", str(line_crop_padding)]

    env = os.environ.copy()
    if ort_lib_dir is not None:
        prev = env.get("DYLD_LIBRARY_PATH", "")
        prefix = str(ort_lib_dir)
        env["DYLD_LIBRARY_PATH"] = f"{prefix}:{prev}" if prev else prefix

    t0 = time.perf_counter()
    try:
        subprocess.run(
            cmd,
            cwd=repo_root,
            check=True,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.PIPE,
            text=True,
            env=env,
        )
    except subprocess.CalledProcessError as e:
        raise SystemExit(f"recognize-page failed ({image}):\n{e.stderr}") from e
    return time.perf_counter() - t0


def parse_args() -> argparse.Namespace:
    root = default_repo_root()
    p = argparse.ArgumentParser(description="Timed recognize-page + CER/WER report (Rust).")
    p.add_argument("--repo-root", type=Path, default=root, help="Repository root (default: auto)")
    p.add_argument("--input-dir", type=Path, help="Input images directory")
    p.add_argument("--truth-dir", type=Path, required=True)
    p.add_argument("--pred-dir", type=Path, required=True, help="Rust prediction output directory")
    p.add_argument("--output-dir", type=Path, required=True)
    p.add_argument("--original-pred-dir", type=Path, default=None, help="Optional ndlocr predictions for comparison")
    p.add_argument("--no-build", action="store_true", help="Do not run cargo build --release --features onnx")
    p.add_argument("--no-run", action="store_true", help="Skip inference; only aggregate from existing pred-dir(s)")
    p.add_argument("--normalize", choices=["none", "basic", "strict"], default="strict")
    p.add_argument("--require-all", action="store_true", help="Fail if any prediction is missing for a truth file")
    p.add_argument(
        "--model",
        type=Path,
        default=root
        / "models"
        / "parseq-ndl-24x768-100-tiny-153epoch-tegaki3-r8data-202604.onnx",
    )
    p.add_argument("--charset", type=Path, default=root / "ndlocr" / "src" / "config" / "NDLmoji.yaml")
    p.add_argument("--binarize-threshold", type=int, default=220)
    p.add_argument("--rule-pack", type=Path, default=None)
    p.add_argument("--line-crop-padding", type=int, default=None)
    p.add_argument("--ort-lib-dir", type=Path, default=None)
    p.add_argument("--binary", type=Path, default=None, help="Override path to ndlocr-lite-rs binary")
    return p.parse_args()


def main() -> int:
    args = parse_args()
    repo_root = args.repo_root.expanduser().resolve()
    truth_dir = args.truth_dir.expanduser().resolve()
    pred_dir = args.pred_dir.expanduser().resolve()
    output_dir = args.output_dir.expanduser().resolve()
    output_dir.mkdir(parents=True, exist_ok=True)

    timing_rows: List[Tuple[str, float]] = []
    ran_inference = False

    if not args.no_run:
        if args.input_dir is None:
            raise SystemExit("--input-dir is required unless --no-run")
        input_dir = args.input_dir.expanduser().resolve()
        images = discover_eval_images(input_dir)
        bin_path = (
            args.binary.expanduser().resolve()
            if args.binary
            else ensure_release_binary(repo_root, no_build=args.no_build)
        )
        pred_dir.mkdir(parents=True, exist_ok=True)
        for image in images:
            stem = image.stem
            out_txt = pred_dir / f"{stem}.txt"
            print(f"[timed] {image.name} -> {out_txt.name}", file=sys.stderr)
            sec = run_recognize_page_timed(
                bin_path=bin_path,
                repo_root=repo_root,
                image=image,
                out_txt=out_txt,
                model=args.model.expanduser().resolve(),
                charset=args.charset.expanduser().resolve(),
                binarize_threshold=args.binarize_threshold,
                rule_pack=args.rule_pack.expanduser().resolve() if args.rule_pack else None,
                line_crop_padding=args.line_crop_padding,
                ort_lib_dir=args.ort_lib_dir.expanduser().resolve() if args.ort_lib_dir else None,
            )
            timing_rows.append((stem, sec))
        ran_inference = True
        write_timing_csv(output_dir / "timing_rust.csv", timing_rows)
    elif (output_dir / "timing_rust.csv").is_file():
        with (output_dir / "timing_rust.csv").open(encoding="utf-8", newline="") as f:
            reader = csv.DictReader(f)
            for row in reader:
                timing_rows.append((row["stem"], float(row["wall_seconds"])))
        ran_inference = True

    truth_files = eom.discover_truth_files(truth_dir)
    if not truth_files:
        raise SystemExit(f"no truth *.txt under {truth_dir}")

    systems: List[eom.SystemMetrics] = []
    systems.append(eom.evaluate_system("rust", pred_dir, truth_dir, truth_files, args.normalize))
    if args.original_pred_dir is not None:
        od = args.original_pred_dir.expanduser().resolve()
        systems.append(eom.evaluate_system("original", od, truth_dir, truth_files, args.normalize))

    baseline = "original" if args.original_pred_dir is not None else None

    if args.require_all:
        for s in systems:
            if s.missing_count:
                print(f"error: system {s.name!r} missing {s.missing_count} files", file=sys.stderr)
                return 2

    eom.write_csv(output_dir / "metrics.csv", systems)
    (output_dir / "metrics.md").write_text(
        eom.build_markdown_report(systems, args.normalize, len(truth_files), baseline_system=baseline),
        encoding="utf-8",
    )
    (output_dir / "report.md").write_text(
        build_combined_report_md(
            normalize=args.normalize,
            truth_count=len(truth_files),
            timing_rows=timing_rows,
            systems=systems,
            baseline_system=baseline,
            ran_inference=ran_inference,
        ),
        encoding="utf-8",
    )

    print(f"wrote {output_dir / 'report.md'}", file=sys.stderr)
    baseline_metrics = next((s for s in systems if s.name == baseline), None)
    if baseline_metrics is not None:
        print("system,matched_files,missing_files,cer,wer,delta_cer_vs_baseline,delta_wer_vs_baseline")
        for s in sorted(systems, key=lambda x: x.cer):
            print(
                f"{s.name},{s.matched_count},{s.missing_count},{s.cer:.6f},{s.wer:.6f},"
                f"{(s.cer - baseline_metrics.cer):+.6f},{(s.wer - baseline_metrics.wer):+.6f}"
            )
    else:
        print("system,matched_files,missing_files,cer,wer")
        for s in sorted(systems, key=lambda x: x.cer):
            print(f"{s.name},{s.matched_count},{s.missing_count},{s.cer:.6f},{s.wer:.6f}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
