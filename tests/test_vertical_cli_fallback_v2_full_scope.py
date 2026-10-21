from pathlib import Path
import sys
import unittest
sys.path.insert(0,str(Path(__file__).resolve().parents[1]/'tools'))


class FVCv2FullScopeTest(unittest.TestCase):
    def frozen(self):
        import json
        from verify_additional_real_baseline import _packet
        folder=Path(__file__).resolve().parents[1]/'docs/benchmarks'
        old=json.loads((folder/'vertical-cli-fallback/full-matrix-v1/bundle.json').read_text())
        _,data=_packet(folder/'vertical-cli-fallback-v2','before-nn-v1.index.json')
        new=json.loads(data['evidence/new-manifest.json'])
        return old,new

    def test_complete_union_preserves_every_old_origin_and_gt_alias(self):
        from vertical_cli_fallback_v2_full_scope import combine_scope
        from vertical_cli_fallback_full_matrix import full_plan
        old,new=self.frozen();images,scores=combine_scope(old,new)
        self.assertEqual(images[:1239],old['physical_inputs'])
        self.assertEqual(scores[:1245],old['scoring_inputs'])
        self.assertEqual((len(images),len(scores)),(1311,1317))
        self.assertEqual(len({(r['width'],r['height'],r['rgb_sha256']) for r in images}),1310)
        self.assertEqual({r['origin'] for r in images[1239:]},{'fvc_v2_new72'})
        self.assertEqual({r['physical_id'] for r in scores},{r['id'] for r in images})
        plan=full_plan(images,scores)
        self.assertEqual((len(plan['jobs']),plan['expected_image_results'],plan['expected_CSV_rows']),(56,26220,79020))

    def test_partial_collision_and_rgb_overlap_cannot_change_scope(self):
        from vertical_cli_fallback_v2_full_scope import combine_scope
        import copy
        old,new=self.frozen()
        bad=copy.deepcopy(new);bad['inputs'].pop()
        with self.assertRaises(ValueError):combine_scope(old,bad)
        bad=copy.deepcopy(new);bad['inputs'][0]['id']=old['physical_inputs'][0]['id']
        with self.assertRaises(ValueError):combine_scope(old,bad)
        bad=copy.deepcopy(new);bad['inputs'][0]['rgb_sha256']=old['physical_inputs'][0]['rgb_sha256']
        with self.assertRaises(ValueError):combine_scope(old,bad)

    def test_alias_only_collisions_and_foreign_physical_references_are_rejected(self):
        from vertical_cli_fallback_v2_full_scope import combine_scope
        import copy
        old,new=self.frozen();physical={r['id'] for r in old['physical_inputs']}
        alias=next(r['id'] for r in old['scoring_inputs'] if r['id'] not in physical)
        bad=copy.deepcopy(new);bad['inputs'][0]['id']=alias
        with self.assertRaises(ValueError):combine_scope(old,bad)
        bad=copy.deepcopy(old);bad['scoring_inputs'][0]['physical_id']='unknown_physical'
        with self.assertRaises(ValueError):combine_scope(bad,new)
        images,scores=combine_scope(old,new)
        images[0]['origin']='changed';scores[0]['truth']='changed'
        self.assertNotEqual(old['physical_inputs'][0]['origin'],'changed')
        self.assertNotEqual(old['scoring_inputs'][0]['truth'],'changed')
