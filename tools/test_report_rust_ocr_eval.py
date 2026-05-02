#!/usr/bin/env python3
"""Tests for report_rust_ocr_eval (no OCR / no cargo)."""

from __future__ import annotations

import tempfile
import unittest
from pathlib import Path
from unittest import mock

import eval_ocr_metrics as eom
import report_rust_ocr_eval as rre


class DiscoverEvalImagesTests(unittest.TestCase):
    def test_finds_png(self) -> None:
        with tempfile.TemporaryDirectory() as d:
            p = Path(d)
            (p / "a.png").write_bytes(b"\x89PNG\r\n\x1a\n")
            (p / "skip.txt").write_text("x")
            found = rre.discover_eval_images(p)
            self.assertEqual([x.name for x in found], ["a.png"])

    def test_empty_errors(self) -> None:
        with tempfile.TemporaryDirectory() as d:
            with self.assertRaises(SystemExit):
                rre.discover_eval_images(Path(d))


class BuildTimingMarkdownTests(unittest.TestCase):
    def test_with_rows(self) -> None:
        md = rre.build_timing_markdown([("a", 1.0), ("b", 2.5)], ran_inference=True)
        self.assertIn("3.500", md)
        self.assertIn("| a |", md)

    def test_no_inference(self) -> None:
        md = rre.build_timing_markdown([], ran_inference=False)
        self.assertIn("計測を行っていません", md)


class ReleaseBinaryPathTests(unittest.TestCase):
    def test_respects_cargo_target_dir(self) -> None:
        with tempfile.TemporaryDirectory() as d:
            with mock.patch.dict("os.environ", {"CARGO_TARGET_DIR": d}):
                self.assertEqual(
                    rre.release_binary_path(Path("/repo")),
                    Path(d).resolve() / "release" / "ndlocr-lite-rs",
                )


class CombinedReportTests(unittest.TestCase):
    def test_combined_contains_accuracy_table(self) -> None:
        systems = [
            eom.SystemMetrics(
                name="rust",
                files=[
                    eom.FileMetrics(
                        relpath="x.txt",
                        cer=0.1,
                        wer=0.2,
                        char_errors=1,
                        char_total=10,
                        word_errors=1,
                        word_total=5,
                    )
                ],
                missing_files=[],
            )
        ]
        md = rre.build_combined_report_md(
            normalize="strict",
            truth_count=1,
            timing_rows=[("scaned0", 1.23)],
            systems=systems,
            baseline_system=None,
            ran_inference=True,
        )
        self.assertIn("# Rust OCR 評価", md)
        self.assertIn("1.230", md)
        self.assertIn("CER", md)


if __name__ == "__main__":
    unittest.main()
