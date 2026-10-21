from pathlib import Path
import copy
import sys
import unittest
sys.path.insert(0,str(Path(__file__).resolve().parents[1]/'tools'))


class RootCompletedV2Test(unittest.TestCase):
    def test_root_native_and_disabled_evidence_replay_without_recognition(self):
        from verify_vertical_cli_fallback_v2_root_completed import verify
        folder=Path(__file__).resolve().parents[1]/'docs/benchmarks/vertical-cli-fallback-v2'
        result=verify(folder)
        self.assertEqual(result['Root_CLI_literal_Green'],2)
        self.assertEqual(result['quality_settings'],3)
        self.assertEqual(result['disabled_CLI_processes'],16)
        self.assertEqual(result['source_files'],852)
        self.assertEqual(result['new_model_calls_in_replay'],0)

    def test_quality_records_reject_skipped_finish_changed_bits_and_nonfinite_values(self):
        from verify_vertical_cli_fallback_v2_root_completed import verify_quality_records
        rows=[{'quality_boost':quality,'quality_boost_min_delta':delta,'bbox':[0,0,48,592],'crop_width':48,'crop_height':592,'initial_text':'春夏秋冬','initial_mean_bits':1065353216,'expected_text':'春夏秋冬','expected_mean_bits':1065353216 if delta>0 else 1065340000,'actual_text':'春夏秋冬','actual_mean_bits':1065353216 if delta>0 else 1065340000} for quality,delta in [(False,0.03),(True,0.03),(True,-100.0)]]
        self.assertEqual(verify_quality_records(rows),3)
        for fault in ['missing','bits','text','no_effect','profile',-1,2**32,0x7fc00000,0x7f800000,0xbf800000]:
            changed=copy.deepcopy(rows)
            if fault=='missing':changed.pop()
            elif fault=='bits':changed[-1]['actual_mean_bits']+=1
            elif fault=='text':changed[-1]['actual_text']='changed'
            elif fault=='no_effect':changed[-1]['actual_mean_bits']=changed[-1]['expected_mean_bits']=changed[-1]['initial_mean_bits']
            elif fault=='profile':changed[-1]['quality_boost_min_delta']=0.03
            else:changed[-1]['initial_mean_bits']=fault
            with self.subTest(fault=fault),self.assertRaises(AssertionError):verify_quality_records(changed)
