from pathlib import Path
import sys
import unittest
sys.path.insert(0, str(Path(__file__).resolve().parents[1] / 'tools'))
from vertical_page_model_choice import classify_model_choice


def page(text, bounds=(2, 3, 22, 243), confidence=.97):
    return {'text':text,'lines':[{'text':text,'bbox':list(bounds),'confidence':confidence}]}


class ModelChoiceTest(unittest.TestCase):
    def test_longer_same_crop_is_only_a_page_proxy_and_records_prefix(self):
        result=classify_model_choice(page('abcde'),page('abc'),['abcde'],3)
        self.assertIsNotNone(result)
        self.assertEqual(result['category'],'shorter_same_crop')
        self.assertTrue(result['model100_is_complete_prefix'])
        self.assertTrue(result['raw_limit_proxy_only'])
        self.assertFalse(result['actual_CLI_observed'])

    def test_different_or_absent_bbox_is_not_a_same_crop_claim(self):
        for other in [page('abc',(2,3,23,243)),{'text':'','lines':[]}]:
            result=classify_model_choice(page('abcde'),other,['abcde'],3)
            self.assertIsNotNone(result)
            self.assertEqual(result['category'],'different_or_absent_crop')
            self.assertFalse(result['model100_is_complete_prefix'])

    def test_same_length_substitution_is_separate_from_shortening(self):
        result=classify_model_choice(page('abcde'),page('abXde'),['abcde'],3)
        self.assertIsNotNone(result)
        self.assertEqual(result['category'],'different_text_same_crop')
        self.assertFalse(result['model100_is_complete_prefix'])

    def test_all_fixed_truth_aliases_must_improve(self):
        self.assertIsNone(classify_model_choice(page('abcde'),page('abc'),['abcde','abc'],3))
        self.assertIsNone(classify_model_choice(page('abcde'),page('abc'),[],3))
        self.assertIsNone(classify_model_choice(page('abcde'),page('abc'),[''],3))

    def test_raw_limit_and_confidence_boundaries(self):
        self.assertIsNone(classify_model_choice(page('abcde'),page('abc'),['abcde'],5))
        for confidence in [.899,float('nan'),float('inf'),1.001,True]:
            self.assertIsNone(classify_model_choice(page('abcde',confidence=confidence),page('abc'),['abcde'],3))
        for confidence in [.9,1.0]:
            self.assertIsNotNone(classify_model_choice(page('abcde',confidence=confidence),page('abc'),['abcde'],3))

    def test_horizontal_or_inconsistent_aggregate_is_not_a_vertical_observation(self):
        self.assertIsNone(classify_model_choice(page('abcde',(0,0,240,20)),page('abc'),['abcde'],3))
        bad=page('abcde');bad['text']='other'
        self.assertIsNone(classify_model_choice(bad,page('abc'),['abcde'],3))


if __name__ == '__main__':
    unittest.main()
