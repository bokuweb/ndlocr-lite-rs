from pathlib import Path
import sys
import unittest
sys.path.insert(0,str(Path(__file__).resolve().parents[1]/'tools'))


class FVCv2PacketTest(unittest.TestCase):
    def test_new_controls_sources_and_both_fresh_build_receipts_authenticate_before_recognition(self):
        from verify_vertical_cli_fallback_v2 import verify_before
        result=verify_before(Path(__file__).resolve().parents[1]/'docs/benchmarks/vertical-cli-fallback-v2')
        self.assertEqual(result['new_inputs'],72)
        self.assertEqual(result['known_inputs'],1239)
        self.assertEqual(result['annotated_glyphs'],4392)
        self.assertEqual(result['hash_overlap'],0)
        self.assertEqual(result['NN_calls_at_freeze'],0)

    def test_known_finish_regression_keeps_failed_attempts_and_real_quality_effect(self):
        from verify_vertical_cli_fallback_v2 import verify_known
        result=verify_known(Path(__file__).resolve().parents[1]/'docs/benchmarks/vertical-cli-fallback-v2')
        self.assertEqual(result['successful_settings'],3)
        self.assertEqual(result['all_recorded_settings'],9)
        self.assertEqual(result['independent_crop_RGBs'],9)
        self.assertEqual(result['preserved_failed_attempts'],3)
        self.assertTrue(result['quality_changes_initial'])
        self.assertFalse(result['new_accuracy_gain'])
