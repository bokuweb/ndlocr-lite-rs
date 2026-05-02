#!/usr/bin/env python3
"""OCR evaluation utility (CER/WER) for multiple systems.

Usage example:
  python tools/eval_ocr_metrics.py \
    --truth-dir tests/fixtures/eval/truth \
    --system rust=tmp/eval/rust \
    --system original=tmp/eval/original \
    --output-md tmp/eval/report.md
"""

from __future__ import annotations

import argparse
import csv
from dataclasses import dataclass
from pathlib import Path
import re
from typing import Iterable, List, Tuple


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Evaluate OCR predictions with CER/WER.")
    parser.add_argument("--truth-dir", required=True, type=Path, help="Ground-truth txt directory")
    parser.add_argument(
        "--system",
        action="append",
        default=[],
        metavar="NAME=DIR",
        help="System name and prediction txt directory (repeatable)",
    )
    parser.add_argument(
        "--normalize",
        choices=["none", "basic", "strict"],
        default="basic",
        help="Text normalization level",
    )
    parser.add_argument(
        "--output-csv",
        type=Path,
        default=None,
        help="Write per-file metrics CSV",
    )
    parser.add_argument(
        "--output-md",
        type=Path,
        default=None,
        help="Write markdown summary report",
    )
    parser.add_argument(
        "--require-all",
        action="store_true",
        help="Fail when any system is missing prediction files for truth set",
    )
    parser.add_argument(
        "--baseline-system",
        type=str,
        default=None,
        help="Optional baseline system name to show delta metrics against",
    )
    return parser.parse_args()


@dataclass
class FileMetrics:
    relpath: str
    cer: float
    wer: float
    char_errors: int
    char_total: int
    word_errors: int
    word_total: int


@dataclass
class SystemMetrics:
    name: str
    files: List[FileMetrics]
    missing_files: List[str]

    @property
    def cer(self) -> float:
        num = sum(f.char_errors for f in self.files)
        den = sum(f.char_total for f in self.files)
        return (num / den) if den > 0 else 0.0

    @property
    def wer(self) -> float:
        num = sum(f.word_errors for f in self.files)
        den = sum(f.word_total for f in self.files)
        return (num / den) if den > 0 else 0.0

    @property
    def matched_count(self) -> int:
        return len(self.files)

    @property
    def missing_count(self) -> int:
        return len(self.missing_files)


def normalize_text(text: str, level: str) -> str:
    if level == "none":
        return text
    t = text.replace("\r\n", "\n").replace("\r", "\n")
    # Keep line boundaries but fold internal whitespace.
    t = "\n".join(" ".join(line.split()) for line in t.split("\n"))
    if level == "strict":
        # Remove ASCII punctuation and spaces for a character-focused check.
        t = re.sub(r"[ !\"#$%&'()*+,\-./:;<=>?@\[\\\]^_`{|}~]", "", t)
    return t.strip()


def levenshtein(a: List[str], b: List[str]) -> int:
    if not a:
        return len(b)
    if not b:
        return len(a)
    if len(a) < len(b):
        a, b = b, a

    prev = list(range(len(b) + 1))
    for i, ca in enumerate(a, start=1):
        cur = [i]
        for j, cb in enumerate(b, start=1):
            cost = 0 if ca == cb else 1
            cur.append(min(cur[j - 1] + 1, prev[j] + 1, prev[j - 1] + cost))
        prev = cur
    return prev[-1]


def discover_truth_files(truth_dir: Path) -> List[Path]:
    return sorted(p for p in truth_dir.rglob("*.txt") if p.is_file())


def parse_system_arg(raw: str) -> Tuple[str, Path]:
    if "=" not in raw:
        raise ValueError(f"invalid --system format: {raw!r}, expected NAME=DIR")
    name, path = raw.split("=", 1)
    if not name.strip():
        raise ValueError(f"invalid system name in {raw!r}")
    return name.strip(), Path(path).expanduser().resolve()


def evaluate_system(
    name: str,
    pred_dir: Path,
    truth_dir: Path,
    truth_files: Iterable[Path],
    normalize_level: str,
) -> SystemMetrics:
    files: List[FileMetrics] = []
    missing_files: List[str] = []
    for truth_path in truth_files:
        rel = truth_path.relative_to(truth_dir).as_posix()
        pred_path = pred_dir / rel
        if not pred_path.is_file():
            missing_files.append(rel)
            continue
        truth_text = normalize_text(truth_path.read_text(encoding="utf-8"), normalize_level)
        pred_text = normalize_text(pred_path.read_text(encoding="utf-8"), normalize_level)

        truth_chars = list(truth_text)
        pred_chars = list(pred_text)
        char_total = max(1, len(truth_chars))
        char_errors = levenshtein(truth_chars, pred_chars)

        truth_words = truth_text.split()
        pred_words = pred_text.split()
        word_total = max(1, len(truth_words))
        word_errors = levenshtein(truth_words, pred_words)

        files.append(
            FileMetrics(
                relpath=rel,
                cer=char_errors / char_total,
                wer=word_errors / word_total,
                char_errors=char_errors,
                char_total=char_total,
                word_errors=word_errors,
                word_total=word_total,
            )
        )
    return SystemMetrics(name=name, files=files, missing_files=missing_files)


def write_csv(path: Path, systems: List[SystemMetrics]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("w", encoding="utf-8", newline="") as f:
        writer = csv.writer(f)
        writer.writerow(["system", "file", "cer", "wer", "char_errors", "char_total", "word_errors", "word_total"])
        for s in systems:
            for m in s.files:
                writer.writerow(
                    [
                        s.name,
                        m.relpath,
                        f"{m.cer:.6f}",
                        f"{m.wer:.6f}",
                        m.char_errors,
                        m.char_total,
                        m.word_errors,
                        m.word_total,
                    ]
                )


def build_markdown_report(
    systems: List[SystemMetrics],
    normalize_level: str,
    truth_count: int,
    baseline_system: str | None = None,
) -> str:
    lines: List[str] = []
    lines.append("# OCR Evaluation Report")
    lines.append("")
    lines.append(f"- normalization: `{normalize_level}`")
    lines.append(f"- truth files discovered: `{truth_count}`")
    if baseline_system:
        lines.append(f"- baseline system: `{baseline_system}`")
    lines.append("")
    lines.append("## Aggregate Metrics")
    lines.append("")
    if baseline_system:
        lines.append("| system | matched_files | missing_files | CER | WER | ΔCER vs baseline | ΔWER vs baseline |")
        lines.append("|---|---:|---:|---:|---:|---:|---:|")
    else:
        lines.append("| system | matched_files | missing_files | CER | WER |")
        lines.append("|---|---:|---:|---:|---:|")
    baseline = next((s for s in systems if s.name == baseline_system), None)
    for s in sorted(systems, key=lambda x: x.cer):
        if baseline is not None:
            dcer = s.cer - baseline.cer
            dwer = s.wer - baseline.wer
            lines.append(
                f"| {s.name} | {s.matched_count} | {s.missing_count} | {s.cer:.4f} | {s.wer:.4f} | {dcer:+.4f} | {dwer:+.4f} |"
            )
        else:
            lines.append(
                f"| {s.name} | {s.matched_count} | {s.missing_count} | {s.cer:.4f} | {s.wer:.4f} |"
            )
    lines.append("")
    lines.append("## Notes")
    lines.append("")
    lines.append("- Lower is better for both CER and WER.")
    lines.append("- Systems are compared only on files that exist in each prediction directory.")
    lines.append("- Use `--require-all` to fail fast when prediction files are missing.")
    lines.append("")
    lines.append("## Missing Files")
    lines.append("")
    for s in systems:
        if not s.missing_files:
            continue
        lines.append(f"### {s.name}")
        lines.append("")
        for rel in s.missing_files[:30]:
            lines.append(f"- {rel}")
        if len(s.missing_files) > 30:
            lines.append(f"- ... ({len(s.missing_files) - 30} more)")
        lines.append("")
    lines.append("")
    return "\n".join(lines)


def main() -> int:
    args = parse_args()
    truth_dir = args.truth_dir.expanduser().resolve()
    if not truth_dir.is_dir():
        raise SystemExit(f"truth dir not found: {truth_dir}")
    if not args.system:
        raise SystemExit("at least one --system NAME=DIR is required")

    truth_files = discover_truth_files(truth_dir)
    if not truth_files:
        raise SystemExit(f"no *.txt files found under truth dir: {truth_dir}")

    systems: List[SystemMetrics] = []
    system_names: List[str] = []
    for raw in args.system:
        name, pred_dir = parse_system_arg(raw)
        if not pred_dir.is_dir():
            raise SystemExit(f"prediction dir not found for system {name}: {pred_dir}")
        system_names.append(name)
        result = evaluate_system(name, pred_dir, truth_dir, truth_files, args.normalize)
        systems.append(result)

    if args.baseline_system and args.baseline_system not in system_names:
        raise SystemExit(
            f"baseline system not found: {args.baseline_system} "
            f"(available: {', '.join(system_names)})"
        )

    if args.output_csv:
        write_csv(args.output_csv, systems)
    if args.output_md:
        args.output_md.parent.mkdir(parents=True, exist_ok=True)
        args.output_md.write_text(
            build_markdown_report(
                systems,
                args.normalize,
                len(truth_files),
                baseline_system=args.baseline_system,
            ),
            encoding="utf-8",
        )

    if args.require_all:
        missing = [s for s in systems if s.missing_count > 0]
        if missing:
            for s in missing:
                print(
                    f"error: system '{s.name}' is missing {s.missing_count} files "
                    f"(first: {s.missing_files[0]})",
                )
            raise SystemExit(2)

    baseline = next((s for s in systems if s.name == args.baseline_system), None)
    if baseline is not None:
        print("system,matched_files,missing_files,cer,wer,delta_cer_vs_baseline,delta_wer_vs_baseline")
        for s in sorted(systems, key=lambda x: x.cer):
            print(
                f"{s.name},{s.matched_count},{s.missing_count},{s.cer:.6f},{s.wer:.6f},"
                f"{(s.cer - baseline.cer):+.6f},{(s.wer - baseline.wer):+.6f}"
            )
    else:
        print("system,matched_files,missing_files,cer,wer")
        for s in sorted(systems, key=lambda x: x.cer):
            print(f"{s.name},{s.matched_count},{s.missing_count},{s.cer:.6f},{s.wer:.6f}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
