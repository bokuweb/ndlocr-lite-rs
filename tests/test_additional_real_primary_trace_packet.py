from pathlib import Path
import sys
import unittest
sys.path.insert(0,str(Path(__file__).resolve().parents[1]/'tools'))


class AdditionalRealPrimaryPacketTest(unittest.TestCase):
    def test_all_fixed_trace_results_pixels_and_alias_scores_replay_without_models(self):
        from verify_additional_real_primary_trace import verify
        result=verify(Path(__file__).resolve().parents[1]/'docs/benchmarks/real-vertical-additional-primary-trace')
        self.assertEqual(result['root_FVB_signatures_exact'],36)
        self.assertEqual(result['all_trace_RGBs'],174)
        self.assertEqual(result['independent_primary_RGB_exact'],36)
        self.assertEqual(result['independent_O_part_RGB_exact'],30)
        self.assertEqual(result['verified_CSV_rows'],396)
        self.assertEqual(result['actual_PARSeq_forward_successes'],66)
        self.assertFalse(result['new_inference_policy'])
