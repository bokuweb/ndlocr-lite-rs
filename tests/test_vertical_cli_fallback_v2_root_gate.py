from pathlib import Path
import sys
import unittest
sys.path.insert(0,str(Path(__file__).resolve().parents[1]/'tools'))


class RootV2GateTest(unittest.TestCase):
    def valid(self):
        return {'complete':True,'full_gate_pass':True,'image_results':26220,'verified_CSV_rows':79020,'regression_pairs':0,'page_body_bbox_confidence_changes':0}

    def test_complete_v2_with_no_regressions_and_identical_pages_can_proceed(self):
        from vertical_cli_fallback_v2_option_runner import require_root_v2_full_gate
        require_root_v2_full_gate(self.valid())

    def test_v1_partial_missing_bad_scores_and_boolean_counts_do_not_proceed(self):
        from vertical_cli_fallback_v2_option_runner import require_root_v2_full_gate
        for change in [{'image_results':24780,'verified_CSV_rows':74700},{'complete':False},{'full_gate_pass':False},{'regression_pairs':1},{'page_body_bbox_confidence_changes':1},{'verified_CSV_rows':79019},{'regression_pairs':False}]:
            with self.subTest(change=change),self.assertRaises(ValueError):require_root_v2_full_gate(self.valid()|change)
        for key in self.valid():
            value=self.valid();del value[key]
            with self.subTest(missing=key),self.assertRaises(ValueError):require_root_v2_full_gate(value)
