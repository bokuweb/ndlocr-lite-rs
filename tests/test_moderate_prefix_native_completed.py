from pathlib import Path
import json
import sys
import unittest
from unittest.mock import patch
sys.path.insert(0,str(Path(__file__).resolve().parents[1]/'tools'))


class ModerateNativeCompletedTest(unittest.TestCase):
    def test_all_native_outputs_and_csv_replay_and_require_hold_without_body_gain(self):
        from verify_moderate_prefix_native_completed import verify
        result=verify(Path(__file__).resolve().parents[1]/'docs/benchmarks/vertical-moderate-prefix')
        self.assertEqual(result['image_results'],3456)
        self.assertEqual(result['verified_CSV_rows'],10368)
        self.assertEqual(result['raw_comparison_pairs'],1728)
        self.assertEqual(result['normalized_comparison_pairs'],5184)
        self.assertEqual(result['regression_pairs'],0)
        self.assertEqual(result['protected_changes'],0)
        self.assertEqual(result['new_body_fragment_auto_gains'],0)
        self.assertEqual(result['meanbit_changes'],0)
        self.assertFalse(result['new72_gate_pass'])
        self.assertEqual(result['NN_calls_in_replay'],0)

    def test_a_success_summary_cannot_hide_a_missing_image_result(self):
        import verify_moderate_prefix_native_completed as reader
        from verify_additional_real_baseline import _packet
        folder=Path(__file__).resolve().parents[1]/'docs/benchmarks/vertical-moderate-prefix';index,data=_packet(folder,'completed-native-v1.index.json');changed=dict(data)
        name='native/candidate-30/results.json';rows=json.loads(changed[name]);rows.pop();changed[name]=json.dumps(rows).encode()
        original=reader._packet
        def packets(target,index_name):
            return (index,changed) if index_name=='completed-native-v1.index.json' else original(target,index_name)
        with patch.object(reader,'_packet',side_effect=packets),self.assertRaises(AssertionError):reader.verify(folder)
