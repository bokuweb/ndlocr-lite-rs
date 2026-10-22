from pathlib import Path
import sys
import unittest
sys.path.insert(0,str(Path(__file__).resolve().parents[1]/'tools'))


class ModerateMatrixScopeTest(unittest.TestCase):
    def test_all_eight_routes_and_three_fragment_auto_routes_have_exact_counts(self):
        from moderate_prefix_matrix_scope import matrix_scope
        from moderate_prefix_controls import control_plan
        result=matrix_scope([r['id'] for r in control_plan(['arial','bold','serif'])])
        self.assertEqual(len(result['routes']),8)
        self.assertEqual(set(result['affected_routes']),{'fragment_auto','fragment_batch_auto','fragment_exclusive_batch_auto'})
        self.assertEqual(len(result['unchanged_routes']),5)
        self.assertFalse(set(result['affected_routes'])&set(result['unchanged_routes']))
        for key,value in {'outer_image_API_calls':2604,'image_results':3456,'CSV_rows':10368,'raw_comparison_pairs':1728,'normalized_comparison_pairs':5184,'explicit_model_loads':18}.items():self.assertEqual(result[key],value)
        self.assertIsNone(result['internal_session_run_calls'])

    def test_missing_duplicate_unknown_or_non_collector_ids_are_rejected(self):
        from moderate_prefix_matrix_scope import matrix_scope
        from moderate_prefix_controls import control_plan
        ids=[r['id'] for r in control_plan(['arial','bold','serif'])]
        for value in [[],ids[:-1],ids+['extra'],ids[:-1]+[ids[0]],ids[:-1]+['unknown_ascii_id'],ids[:-1]+['空'],ids[:-1]+['../../escape'],ids[:-1]+[[]]]:
            with self.subTest(value=value[-1:]),self.assertRaises(ValueError):matrix_scope(value)
