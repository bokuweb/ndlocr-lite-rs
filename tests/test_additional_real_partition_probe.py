from pathlib import Path
import sys
import unittest
sys.path.insert(0,str(Path(__file__).resolve().parents[1]/'tools'))


class AdditionalPartitionProbeTest(unittest.TestCase):
    def test_every_saved_part_and_score_replays_without_loosening_raw_preservation(self):
        from verify_additional_real_partition_probe import verify
        result=verify(Path(__file__).resolve().parents[1]/'docs/benchmarks/real-vertical-primary-partition-probe')
        self.assertEqual(result['raw_API_calls'],117)
        self.assertEqual(result['primary_model_RGB_inputs'],18)
        self.assertEqual(result['original_Page_routes'],36)
        self.assertEqual(result['independent_crop_pixels'],4224384)
        self.assertEqual(result['verified_CSV_rows'],297)
        self.assertEqual(result['model30_agreeing_partitions'],3)
        self.assertEqual(result['model30_agreement_with_full_raw_prefix_extension'],0)
        self.assertEqual(result['new_model_calls_in_replay'],0)
        self.assertFalse(result['new_accuracy_gain'])

    def test_literal_subsequence_keeps_whitespace_repetitions_and_glyph_variants(self):
        from verify_additional_real_partition_probe import is_subsequence
        for raw,text,expected in [('', '甲',True),('甲 ', '甲乙 ',True),('甲乙','乙甲',False),('甲甲','甲',False),(' ', '甲',False),('呉','吳',False)]:
            with self.subTest(raw=raw,text=text):self.assertEqual(is_subsequence(raw,text),expected)
