from pathlib import Path
import sys
import unittest
sys.path.insert(0,str(Path(__file__).resolve().parents[1]/'tools'))


class ModerateNativeMetricsTest(unittest.TestCase):
    def test_gain_is_insufficient_when_a_protected_route_or_nonbody_changes(self):
        from moderate_prefix_native_metrics import pair_decision
        before={'text':'AB','mean_bits':1065353216};after={'text':'ABC','mean_bits':1065353216}
        body=pair_decision(before,after,'ABC','fragment_auto','body')
        self.assertTrue(body['raw_body_gain']);self.assertTrue(body['protected_unchanged']);self.assertFalse(body['regression'])
        self.assertFalse(pair_decision(before,after,'ABC','raw_auto','body')['protected_unchanged'])
        self.assertFalse(pair_decision(before,after,'ABC','fragment_auto','repeated')['protected_unchanged'])
        self.assertTrue(pair_decision(after,before,'ABC','fragment_auto','body')['regression'])
        same=pair_decision(before,before,'ABC','fragment_auto','body');self.assertFalse(same['raw_body_gain']);self.assertTrue(same['body_unchanged'])
        changed_mean={'text':'AB','mean_bits':1065353215}
        self.assertFalse(pair_decision(before,changed_mean,'ABC','normal_auto','body')['protected_unchanged'])

    def test_invalid_confidence_or_unknown_scope_is_never_accepted_as_unchanged(self):
        from moderate_prefix_native_metrics import pair_decision
        value={'text':'AB','mean_bits':1065353216}
        for bad in [{'error':'model failure'},{'text':'AB','mean_bits':0x7fc00000},{'text':'AB','mean_bits':0x7f800000},{'text':'AB','mean_bits':-1},{'text':'AB','mean_bits':True}]:
            with self.subTest(bad=bad),self.assertRaises(ValueError):pair_decision(bad,value,'AB','raw_auto','body')
        for route,cohort in [('unknown','body'),('raw_auto','unknown')]:
            with self.assertRaises(ValueError):pair_decision(value,value,'AB',route,cohort)
