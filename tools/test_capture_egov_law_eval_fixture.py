#!/usr/bin/env python3
"""Unit tests for capture_egov_law_eval_fixture (no network, no Playwright)."""

from __future__ import annotations

import unittest

from capture_egov_law_eval_fixture import derive_fixture_stem, postprocess_truth_text


class DeriveFixtureStemTests(unittest.TestCase):
    def test_law_path(self) -> None:
        self.assertEqual(
            derive_fixture_stem("https://laws.e-gov.go.jp/law/129AC0000000089", None),
            "law_129AC0000000089",
        )

    def test_strips_fragment_and_query(self) -> None:
        self.assertEqual(
            derive_fixture_stem(
                "https://laws.e-gov.go.jp/law/129AC0000000089?foo=1#anchor",
                None,
            ),
            "law_129AC0000000089",
        )

    def test_override(self) -> None:
        self.assertEqual(
            derive_fixture_stem("https://example.com/", "My Sample!!"),
            "My_Sample",
        )


class PostprocessTruthTextTests(unittest.TestCase):
    def test_collapses_internal_space(self) -> None:
        self.assertEqual(postprocess_truth_text("a  \tb\r\nc"), "a b\nc")

    def test_trims_lines(self) -> None:
        self.assertEqual(postprocess_truth_text("  hello  \n  world  "), "hello\nworld")

    def test_collapses_blank_runs(self) -> None:
        self.assertEqual(
            postprocess_truth_text("a\n\n\n\nb"),
            "a\n\nb",
        )


if __name__ == "__main__":
    unittest.main()
