from pathlib import Path
import copy
import sys
import unittest
sys.path.insert(0,str(Path(__file__).resolve().parents[1]/'tools'))
from additional_real_diagnostic import image_manifest


class AdditionalRealManifestTest(unittest.TestCase):
    def setUp(self):
        self.master={'GT_frozen':True,'inputs':[{'id':'a','image':'images/a.png','truth':'NEVER_TO_NN'},{'id':'b','image':'images/b.png'}],
                     'scoring_inputs':[{'id':'a_old','physical_id':'a'},{'id':'a_jis','physical_id':'a'},{'id':'b_only','physical_id':'b'}]}

    def test_each_physical_image_is_recognized_once_and_truth_fields_are_omitted(self):
        before=copy.deepcopy(self.master)
        self.assertEqual(image_manifest(self.master,Path('/work')),{'inputs':[{'id':'a','image':'/work/images/a.png'},{'id':'b','image':'/work/images/b.png'}]})
        self.assertEqual(self.master,before)

    def test_unfrozen_or_duplicate_or_missing_physical_controls_are_rejected(self):
        changes=[{'GT_frozen':False},{'inputs':[]},{'inputs':[self.master['inputs'][0],self.master['inputs'][0]]},
                 {'inputs':[{'id':'../unsafe','image':'images/a.png'}]}]
        for change in changes:
            with self.subTest(change=change),self.assertRaises(ValueError):image_manifest(dict(self.master,**change),Path('/work'))

    def test_missing_duplicate_or_unknown_scoring_aliases_do_not_silently_change_sampling(self):
        changes=[[],[{'id':'alias','physical_id':'unknown'}],
                 [{'id':'alias','physical_id':'a'},{'id':'alias','physical_id':'b'}],
                 [{'id':'alias','physical_id':'a'}]]
        for scores in changes:
            with self.subTest(scores=scores),self.assertRaises(ValueError):image_manifest(dict(self.master,scoring_inputs=scores),Path('/work'))

class TextDiagnosisTest(unittest.TestCase):
    def test_nfc_diagnostic_reports_canonical_change_without_rewriting_raw_errors(self):
        from additional_real_diagnostic import text_diagnosis
        result=text_diagnosis('突',' 突')
        self.assertEqual(result['raw'],{'char_errors':2,'char_total':1,'word_errors':1,'word_total':1})
        self.assertEqual(result['nfc_diagnostic_only']['char_errors'],1)
        self.assertEqual(result['prediction_scalar_count'],2)
        self.assertEqual(result['prediction_ascii_spaces'],1)
        self.assertEqual(result['prediction_nfc_changes'],[{'scalar':'突','codepoint':'U+FA55','nfc':'突'}])
        self.assertFalse(result['adoption_comparison'])

    def test_distinct_historical_glyphs_and_fullwidth_forms_are_not_folded(self):
        from additional_real_diagnostic import text_diagnosis
        result=text_diagnosis('呉德Ａ','吳徳A')
        self.assertEqual(result['raw']['char_errors'],3)
        self.assertEqual(result['nfc_diagnostic_only'],result['raw'])
        self.assertEqual(result['prediction_nfc_changes'],[])

    def test_multiscalar_canonical_composition_is_applied_to_whole_strings(self):
        from additional_real_diagnostic import text_diagnosis
        result=text_diagnosis('が','か\u3099')
        self.assertEqual(result['raw']['char_errors'],2)
        self.assertEqual(result['nfc_diagnostic_only']['char_errors'],0)
        self.assertEqual(result['prediction_scalar_count'],2)


if __name__=='__main__':unittest.main()
