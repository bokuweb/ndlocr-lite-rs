from pathlib import Path
import sys
import unittest
sys.path.insert(0,str(Path(__file__).resolve().parents[1]/'tools'))


class AdditionalRealPacketTest(unittest.TestCase):
    def test_public_packet_replays_all_alias_metrics_without_models_or_fonts(self):
        from verify_additional_real_baseline import verify
        result=verify(Path(__file__).resolve().parents[1]/'docs/benchmarks/real-vertical-additional-controls')
        self.assertEqual(result['image_results'],48)
        self.assertEqual(result['verified_CSV_rows'],264)
        self.assertEqual(result['verified_raw_diagnoses'],88)
        self.assertFalse(result['adoption_comparison'])
