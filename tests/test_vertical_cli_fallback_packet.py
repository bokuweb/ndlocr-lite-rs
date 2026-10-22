from pathlib import Path
import sys
import unittest
sys.path.insert(0,str(Path(__file__).resolve().parents[1]/'tools'))


class FullFvcNewControlsPacketTest(unittest.TestCase):
    def test_all_saved_72_by_four_routes_replay_and_keep_the_full_gate_pending(self):
        from verify_vertical_cli_fallback import verify
        result=verify(Path(__file__).resolve().parents[1]/'docs/benchmarks/vertical-cli-fallback')
        self.assertEqual(result['image_results'],576)
        self.assertEqual(result['verified_CSV_rows'],1728)
        self.assertEqual(result['new_body_raw_gain_pairs'],30)
        self.assertEqual(result['new_body_physical_images'],14)
        self.assertEqual(result['regression_pairs'],0)
        self.assertTrue(result['new72_gate_pass'])
        self.assertFalse(result['full_gate_pass'])
