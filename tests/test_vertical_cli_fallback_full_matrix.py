from pathlib import Path
import copy
import sys
import unittest
sys.path.insert(0,str(Path(__file__).resolve().parents[1]/'tools'))


class FullFvcContractsTest(unittest.TestCase):
    def groups(self):
        return [{'origin':'base','inputs':[{'id':'a','rgb_sha256':'same'},{'id':'duplicate_pixels','rgb_sha256':'same'}],
                 'scoring_inputs':[{'id':'a_old','physical_id':'a'},{'id':'a_jis','physical_id':'a'},{'id':'dup','physical_id':'duplicate_pixels'}]},
                {'origin':'extra','inputs':[{'id':'b','rgb_sha256':'other'}], 'scoring_inputs':[{'id':'b_gt','physical_id':'b'}]}]

    def test_merge_keeps_all_physical_ids_rgb_duplicates_and_scoring_aliases(self):
        from vertical_cli_fallback_full_matrix import merge_controls
        groups=self.groups();before=copy.deepcopy(groups);images,scores=merge_controls(groups)
        self.assertEqual([r['id'] for r in images],['a','duplicate_pixels','b'])
        self.assertEqual([r['id'] for r in scores],['a_old','a_jis','dup','b_gt'])
        self.assertEqual([r['origin'] for r in images],['base','base','extra'])
        self.assertEqual(groups,before)

    def test_missing_or_duplicate_or_unsafe_ids_and_alias_targets_are_rejected(self):
        from vertical_cli_fallback_full_matrix import merge_controls
        for change in ['duplicate_physical','duplicate_alias','unknown_target','missing_score','unsafe']:
            groups=self.groups()
            if change=='duplicate_physical':groups[1]['inputs'][0]['id']='a'
            if change=='duplicate_alias':groups[1]['scoring_inputs'][0]['id']='a_old'
            if change=='unknown_target':groups[1]['scoring_inputs'][0]['physical_id']='missing'
            if change=='missing_score':groups[0]['scoring_inputs']=groups[0]['scoring_inputs'][:2]
            if change=='unsafe':groups[1]['inputs'][0]['id']='../b'
            with self.subTest(change=change),self.assertRaises(ValueError):merge_controls(groups)

    def test_plan_covers_seven_kinds_and_both_variants_without_losing_page_settings(self):
        from vertical_cli_fallback_full_matrix import full_plan
        plan=full_plan([{'id':str(i)} for i in range(5)],[{'id':'s'+str(i)} for i in range(7)],2)
        self.assertEqual(len(plan['jobs']),14)
        self.assertEqual(plan['expected_image_results'],100)
        self.assertEqual(plan['expected_CSV_rows'],420)
        for kind in plan['kinds']:
            jobs=[r for r in plan['jobs'] if r['kind']==kind]
            self.assertEqual([r['physical_ids'] for r in jobs],[['0','2','4'],['1','3']])

    def test_full_cli_argv_adds_only_the_declared_cascade_switch(self):
        from vertical_cli_fallback_full_matrix import full_cli_command
        from vertical_page_matrix import cli_command
        assets={k:{'name':k+'.onnx'} for k in ['deim','100','30','50','charset']}
        for mode in ['deim','bands']:
            for switch,enabled in [('on','true'),('off','false')]:
                self.assertEqual(full_cli_command('exe',Path('/input.png'),Path('/models'),assets,'cli_'+mode+'_'+switch,Path('/body.txt')),
                                 cli_command('exe',Path('/input.png'),Path('/models'),assets,mode,Path('/body.txt'))+['--enable-cascade',enabled])
        with self.assertRaises(ValueError):full_cli_command('exe',Path('/input'),Path('/models'),assets,'unknown',Path('/body'))

class FullFvcCompletionTest(unittest.TestCase):
    def setUp(self):
        from vertical_cli_fallback_full_matrix import full_plan
        self.plan=full_plan([{'id':str(i)} for i in range(5)],[{'id':'s'+str(i)} for i in range(7)],2)
        self.metas=[{'kind':j['kind'],'complete':True,'shard':{k:j[k] for k in ['index','total','physical_ids']},
                     'bundle_sha256':'bundle','build_receipt_sha256':'build','frozen_materials_after_unchanged':True,
                     'image_results':len(j['physical_ids'])*(4 if j['kind'].startswith('page') else 2)} for j in self.plan['jobs']]

    def test_complete_cartesian_set_is_required(self):
        from vertical_cli_fallback_full_matrix import validate_complete_jobs
        self.assertEqual(validate_complete_jobs(self.metas,self.plan,'bundle','build'),100)
        for change in ['missing','duplicate','partial','wrong_ids','wrong_source','wrong_build','wrong_count','unstamped']:
            metas=copy.deepcopy(self.metas)
            if change=='missing':metas.pop()
            if change=='duplicate':metas.append(copy.deepcopy(metas[0]))
            if change=='partial':metas[0]['complete']=False
            if change=='wrong_ids':metas[0]['shard']['physical_ids']=['unknown']
            if change=='wrong_source':metas[0]['bundle_sha256']='other'
            if change=='wrong_build':metas[0]['build_receipt_sha256']='other'
            if change=='wrong_count':metas[0]['image_results']-=1
            if change=='unstamped':metas[0]['frozen_materials_after_unchanged']=False
            with self.subTest(change=change),self.assertRaises(ValueError):validate_complete_jobs(metas,self.plan,'bundle','build')

class FrozenFullFvcBundleTest(unittest.TestCase):
    def test_entire_union_and_source_bridge_are_bound_before_model_calls(self):
        from vertical_cli_fallback_full_matrix import FullBundle
        from vertical_page_matrix import sha
        import tempfile
        bundle=FullBundle(Path(__file__).resolve().parents[1]/'docs/benchmarks/vertical-cli-fallback/full-matrix-v1')
        self.assertEqual(bundle.index['expected']['physical_inputs'],1239)
        self.assertEqual(bundle.index['expected']['scoring_inputs'],1245)
        self.assertEqual(bundle.plan['expected_CSV_rows'],74700)
        self.assertEqual(len(bundle.plan['jobs']),56)
        self.assertTrue(bundle.baseline_FVB_bridge)
        with tempfile.TemporaryDirectory() as directory:
            selected=bundle.index['physical_inputs'][:2]
            manifest=bundle.materialize_inputs(Path(directory),selected)
            self.assertEqual([r['id'] for r in manifest['inputs']],[r['id'] for r in selected])
            for actual,expected in zip(manifest['inputs'],selected):
                self.assertEqual(set(actual),{'id','image'})
                self.assertEqual(sha(Path(actual['image']).read_bytes()),expected['image_sha256'])

class FullFvcRetrySelectionTest(unittest.TestCase):
    def test_partial_attempts_remain_evidence_and_only_one_complete_current_build_is_used(self):
        from vertical_cli_fallback_full_matrix import select_complete_jobs
        current={'complete':True,'kind':'page30','shard':{'index':0},'bundle_sha256':'bundle','build_receipt_sha256':'build'}
        partial=dict(current,complete=False);old=dict(current,build_receipt_sha256='older')
        selected,excluded=select_complete_jobs([('partial',partial),('old',old),('good',current)],'bundle','build')
        self.assertEqual(selected,[('good',current)])
        self.assertEqual([(r['path'],r['reason']) for r in excluded],[('partial','partial'),('old','different_build_or_bundle')])
        with self.assertRaises(ValueError):select_complete_jobs([('a',current),('b',copy.deepcopy(current))],'bundle','build')

class FullFvcBeforeNnCommandsTest(unittest.TestCase):
    def setUp(self):
        self.assets={k:{'name':k+'.onnx'} for k in ['deim','100','30','50','charset']}
        self.binaries={v:{'cli':'/frozen/'+v+'-cli','page':'/frozen/'+v+'-page'} for v in ['baseline','candidate']}
        self.rows=[{'id':'a','image':'archive/a.png','truth':'NEVER_TO_NN'},{'id':'b','image':'archive/b.jpg','truth':'ALSO_SECRET'}]

    def test_page_plan_declares_both_models_and_only_images_manifest(self):
        from vertical_cli_fallback_full_matrix import worker_commands
        plans=worker_commands(self.assets,self.binaries,self.rows,'page50',Path('/work'),Path('/evidence'),Path('/models'))
        self.assertEqual(len(plans),2)
        for p in plans:
            self.assertEqual(p['argv'][0],self.binaries[p['variant']]['page'])
            self.assertEqual(p['argv'][p['argv'].index('--parseq-model')+1],'/models/50.onnx')
            self.assertEqual(p['argv'][p['argv'].index('--manifest')+1],'/work/NN-images-only.json')
            self.assertNotIn('NEVER_TO_NN',' '.join(p['argv']))

    def test_cli_plan_has_exactly_one_command_per_variant_and_image(self):
        from vertical_cli_fallback_full_matrix import worker_commands
        plans=worker_commands(self.assets,self.binaries,self.rows,'cli_bands_off',Path('/work'),Path('/evidence'),Path('/models'))
        self.assertEqual([(p['variant'],p['id']) for p in plans],[('baseline','a'),('baseline','b'),('candidate','a'),('candidate','b')])
        for p in plans:
            self.assertEqual(p['argv'][p['argv'].index('--image')+1],{'a':'/work/inputs/a.png','b':'/work/inputs/b.jpg'}[p['id']])
            self.assertEqual(p['argv'][-2:],['--enable-cascade','false'])
            self.assertNotIn('SECRET',' '.join(p['argv']))


class FullFvcImageFormatRegressionTest(unittest.TestCase):
    def test_jpeg_and_png_bytes_keep_their_original_loader_extensions(self):
        from vertical_cli_fallback_full_matrix import FullBundle
        import tempfile
        bundle=FullBundle(Path(__file__).resolve().parents[1]/'docs/benchmarks/vertical-cli-fallback/full-matrix-v1')
        selected=[next(r for r in bundle.index['physical_inputs'] if Path(r['image']).suffix==extension) for extension in ['.png','.jpg']]
        with tempfile.TemporaryDirectory() as directory:
            manifest=bundle.materialize_inputs(Path(directory),selected)
            for actual,original in zip(manifest['inputs'],selected):
                path=Path(actual['image'])
                self.assertEqual(path.suffix,Path(original['image']).suffix)
                self.assertEqual(path.read_bytes(),bundle.read_image(original))

    def test_every_staged_file_matches_its_frozen_hash_and_magic_signature(self):
        from vertical_cli_fallback_full_matrix import FullBundle
        from vertical_page_matrix import sha
        import tempfile
        bundle=FullBundle(Path(__file__).resolve().parents[1]/'docs/benchmarks/vertical-cli-fallback/full-matrix-v1')
        with tempfile.TemporaryDirectory() as directory:
            manifest=bundle.materialize_inputs(Path(directory),bundle.index['physical_inputs'])
            signatures={'.png':b'\x89PNG\r\n\x1a\n','.jpg':b'\xff\xd8\xff'}
            counts={'.png':0,'.jpg':0}
            for actual,row in zip(manifest['inputs'],bundle.index['physical_inputs']):
                path=Path(actual['image']);raw=path.read_bytes()
                self.assertEqual(sha(raw),row['image_sha256'])
                self.assertTrue(raw.startswith(signatures[path.suffix]),row['id'])
                counts[path.suffix]+=1
            self.assertEqual(counts,{'.png':1121,'.jpg':118})
