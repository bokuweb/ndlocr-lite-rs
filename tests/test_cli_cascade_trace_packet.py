from pathlib import Path
import sys
import unittest
sys.path.insert(0,str(Path(__file__).resolve().parents[1]/'tools'))


class CliTracePacketTest(unittest.TestCase):
    def test_saved_linux_observation_replays_without_models(self):
        from verify_cli_cascade_trace import verify
        result=verify(Path(__file__).resolve().parents[1]/'docs/benchmarks/vertical-cli-cascade-trace')
        self.assertEqual(result['CLI_processes'],18)
        self.assertEqual(result['verified_CSV_rows'],54)
        self.assertEqual(result['verified_crop_instances'],27)
        self.assertEqual(result['observed_nonempty_pool_batches'],9)
        self.assertEqual(result['default_nonempty_initial100'],3)
        self.assertEqual(result['default_nonempty_promotions'],0)
        self.assertFalse(result['adoption_comparison'])
