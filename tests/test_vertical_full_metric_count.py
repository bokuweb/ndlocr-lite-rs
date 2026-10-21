from pathlib import Path
import sys
import unittest
sys.path.insert(0,str(Path(__file__).resolve().parents[1]/'tools'))


class FullMetricCountTest(unittest.TestCase):
    def test_v1_and_v2_use_their_own_frozen_plan_totals(self):
        from vertical_cli_fallback_full_matrix import require_complete_metric_count
        for expected in [74700,79020]:require_complete_metric_count(expected,{'expected_CSV_rows':expected})

    def test_missing_extra_and_empty_scopes_cannot_pass(self):
        from vertical_cli_fallback_full_matrix import require_complete_metric_count
        for actual,expected in [(74700,79020),(79019,79020),(79021,79020),(0,0),(-1,-1)]:
            with self.subTest(actual=actual,expected=expected),self.assertRaises(ValueError):require_complete_metric_count(actual,{'expected_CSV_rows':expected})
