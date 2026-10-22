from pathlib import Path
import sys
import unittest
sys.path.insert(0,str(Path(__file__).resolve().parents[1]/'tools'))


class ModerateNativeBeforeTest(unittest.TestCase):
    def test_complete_sources_fresh_artifacts_and_no_truth_in_recognition_commands(self):
        from verify_moderate_prefix_native_before import verify
        result=verify(Path(__file__).resolve().parents[1]/'docs/benchmarks/vertical-moderate-prefix')
        self.assertEqual(result['baseline_source_files'],852)
        self.assertEqual(result['candidate_source_files'],855)
        self.assertEqual(result['inputs'],72)
        self.assertEqual(result['planned_image_results'],3456)
        self.assertEqual(result['planned_commands'],6)
        self.assertEqual(result['NN_calls_in_replay'],0)
