from pathlib import Path
import sys
import unittest
sys.path.insert(0,str(Path(__file__).resolve().parents[1]/'tools'))


class VerticalCliFallbackV2FullCompletedTest(unittest.TestCase):
    def test_all_workers_and_every_score_replay_from_the_complete_v2_bundle(self):
        from verify_vertical_cli_full_completed import verify
        from vertical_cli_fallback_v2_full_matrix import FullBundleV2
        folder=Path(__file__).resolve().parents[1]/'docs/benchmarks/vertical-cli-fallback-v2/full-matrix-v1'
        result=verify(folder,FullBundleV2(folder))
        self.assertEqual(result['complete_workers'],56)
        self.assertEqual(result['image_results'],26220)
        self.assertEqual(result['verified_CSV_rows'],79020)
        self.assertEqual(result['scoring_comparison_pairs'],39510)
        self.assertEqual(result['regression_pairs'],0)
        self.assertEqual(result['page_body_bbox_confidence_changes'],0)
        self.assertTrue(result['full_gate_pass'])
        self.assertEqual(result['new_model_calls_in_replay'],0)
