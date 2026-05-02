#!/usr/bin/env python3
"""Tests for compare_rust_original_eval (no OCR)."""

from __future__ import annotations

import unittest
from pathlib import Path

import compare_rust_original_eval as cr
import eval_ocr_metrics as eom


class CompareReportTests(unittest.TestCase):
    def test_resolve_python_bin_relative_to_repo_root(self) -> None:
        resolved = cr.resolve_python_bin("./.venv/bin/python", Path("/repo"))
        self.assertEqual(resolved, "/repo/.venv/bin/python")

    def test_report_has_both_timings(self) -> None:
        systems = [
            eom.SystemMetrics(
                name="rust",
                files=[
                    eom.FileMetrics(
                        relpath="x.txt",
                        cer=0.03,
                        wer=0.1,
                        char_errors=1,
                        char_total=10,
                        word_errors=1,
                        word_total=10,
                    )
                ],
                missing_files=[],
            ),
            eom.SystemMetrics(
                name="original",
                files=[
                    eom.FileMetrics(
                        relpath="x.txt",
                        cer=0.05,
                        wer=0.1,
                        char_errors=1,
                        char_total=10,
                        word_errors=1,
                        word_total=10,
                    )
                ],
                missing_files=[],
            ),
        ]
        md = cr.build_compare_report_md(
            normalize="strict",
            truth_count=1,
            rust_timing=[("x", 2.0)],
            orig_timing=[("x", 5.0)],
            ran_rust=True,
            ran_orig=True,
            systems=systems,
        )
        self.assertIn("Rust vs NDLOCR-Lite", md)
        self.assertIn("ocr.py", md)
        self.assertIn("Aggregate Metrics", md)


if __name__ == "__main__":
    unittest.main()
