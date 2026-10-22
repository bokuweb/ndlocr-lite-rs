import json
from pathlib import Path
import sys
import tempfile
from types import SimpleNamespace
import unittest
from unittest import mock
sys.path.insert(0,str(Path(__file__).resolve().parents[1]/'tools'))
import vertical_page_matrix as matrix
from vertical_page_sharded_matrix import run_shard,verify_shards

class ProducerTests(unittest.TestCase):
    def setUp(self):
        self.temp=tempfile.TemporaryDirectory();self.root=Path(self.temp.name)
        self.inputs=[{'id':str(i)} for i in range(9)]
        self.bundle=SimpleNamespace(index={'physical_inputs':self.inputs,'assets':{'x':'fixed'}},folder=self.root)
        (self.root/'bundle.json').write_text(json.dumps(self.bundle.index))
    def tearDown(self):self.temp.cleanup()
    def fake_runner(self,bundle,kind,output,models):
        plan=json.loads(Path(output).with_name(Path(output).name+'-plan.json').read_text())
        self.assertEqual(plan['shard']['physical_ids'],['0','4','8'])
        self.assertEqual(plan['bundle_sha256'],matrix.sha((self.root/'bundle.json').read_bytes()))
        self.assertEqual([r['id'] for r in bundle.index['physical_inputs']],['0','4','8'])
        self.assertEqual(bundle.index['assets'],{'x':'fixed'})
        output=Path(output);output.mkdir()
        (output/'evidence').mkdir()
        (output/'evidence/job.json').write_text(json.dumps({'kind':kind,'physical_inputs':3,'complete':True}))
    def test_producer_uses_both_frozen_sources_and_only_the_declared_input_subset(self):
        output=self.root/'run'
        with mock.patch.object(matrix,'run_kind',side_effect=self.fake_runner) as run:
            run_shard(self.bundle,'page100',output,self.root/'models',0,4)
            run.assert_called_once()
        meta=json.loads((output/'evidence/job.json').read_text())
        self.assertEqual(meta['shard'],{'index':0,'total':4,'physical_ids':['0','4','8']})
        self.assertEqual(meta['global_physical_inputs'],9)
        self.assertEqual(self.bundle.index['physical_inputs'],self.inputs)
    def test_failed_attempt_retains_shard_identity_and_cannot_become_complete(self):
        output=self.root/'run'
        def failing(bundle,kind,output,models):
            self.fake_runner(bundle,kind,output,models)
            p=output/'evidence/job.json';meta=json.loads(p.read_text());meta['complete']=False;p.write_text(json.dumps(meta))
            raise RuntimeError('recognition failed')
        with mock.patch.object(matrix,'run_kind',side_effect=failing),self.assertRaises(RuntimeError):
            run_shard(self.bundle,'page100',output,self.root/'models',0,4)
        meta=json.loads((output/'evidence/job.json').read_text())
        self.assertFalse(meta['complete']);self.assertEqual(meta['shard']['index'],0)

class AggregateTests(unittest.TestCase):
    def setUp(self):
        import test_vertical_page_matrix as fixture_module
        self.fixture=fixture_module.MatrixTests();self.fixture.setUp();self.root=self.fixture.root
        index=self.fixture.bundle();one=index['physical_inputs'][0]
        index['physical_inputs']=[dict(one,id=name) for name in ['one','two','three','four']]
        template=index['scoring_inputs'][0]
        index['scoring_inputs'] += [dict(template,id=name,physical_id=name) for name in ['two','three','four']]
        index['expected'].update(physical_inputs=4,scoring_inputs=5)
        self.fixture.write_index(index);self.bundle=matrix.Bundle(self.root)
    def tearDown(self):self.fixture.tearDown()
    def artifact(self,root,kind,index=None,total=None):
        from vertical_page_shards import partition_inputs
        selected=partition_inputs(self.bundle.index['physical_inputs'],index,total)
        name=kind if index is None else kind+'-'+str(index)
        folder=root/name;(folder/'commands').mkdir(parents=True)
        original=self.root/'original-runner'/name/'work';evidence=original.parent/'evidence'
        manifest={'inputs':[{'id':r['id'],'image':str(original/r['image'])} for r in selected]}
        matrix.write_json(folder/'NN-images-only.json',manifest)
        meta={'kind':kind,'complete':True,'bundle_sha256':matrix.sha((self.root/'bundle.json').read_bytes()),
              'physical_inputs':len(selected),'assets':self.bundle.index['assets'],'models_dir':str(self.root/'original-models'),
              'all_frozen_materials_unchanged':True,'NN_manifest_sha256':matrix.sha((folder/'NN-images-only.json').read_bytes()),
              'builds':{},'commands':[{'exit_code':0}]*2}
        if index is not None:meta.update(global_physical_inputs=4,shard={'index':index,'total':total,'physical_ids':[r['id'] for r in selected]})
        for variant in ['baseline','candidate']:
            source_manifest=str(original/variant/'Cargo.toml');events=self.fixture.artifacts()
            for event in events:event['manifest_path']=source_manifest
            log=folder/'commands'/(variant+'-build.stdout');log.write_text('\n'.join(json.dumps(x) for x in events))
            meta['builds'][variant]={'source_manifest':source_manifest,'build_stdout_sha256':matrix.sha(log.read_bytes()),
                'source_files':self.bundle.index['source_files'][variant],'artifacts':matrix.check_build_artifacts(events,source_manifest),
                'binaries':{'page':{'sha256':('a' if variant=='baseline' else 'c')*64,'path':str(original/'binaries'/(variant+'-page'))},
                            'cli':{'sha256':'b'*64,'path':str(original/'binaries'/(variant+'-cli'))}}}
            assets=self.bundle.index['assets'];models=Path(meta['models_dir'])
            if kind.startswith('page'):
                for mode in ['eval','default']:
                    directory=folder/'page'/variant/('runtime'+kind[4:]+'_'+mode);directory.mkdir(parents=True)
                    text='wrong' if variant=='baseline' else ' 本文　';rows=[]
                    for inp in selected:
                        (directory/(inp['id']+'.txt')).write_bytes(text.encode())
                        rows.append({'id':inp['id'],'page_index':0,'text':text,'lines':[{'bbox':[0,0,1,1],'text':text,'confidence':0.95}]})
                    matrix.write_json(directory/'results.json',rows)
                model=kind[4:]
                argv=[str(original/'binaries'/(variant+'-page')),'--manifest',str(original/'NN-images-only.json'),
                      '--det-model',str(models/assets['deim']['name']),'--parseq-model',str(models/assets[model]['name']),
                      '--charset',str(models/assets['charset']['name']),'--label','runtime'+model,'--output-dir',str(evidence/'page'/variant)]
                receipt={'argv':argv,'exit_code':0};matrix.write_json(folder/'commands'/(variant+'-page'+model+'.json'),receipt);meta['commands'].append(receipt)
            else:
                for mode in ['deim','bands']:
                    directory=folder/'cli'/variant/mode;directory.mkdir(parents=True)
                    for inp in selected:
                        ident=inp['id'];(directory/(ident+'.txt')).write_bytes(b'wrong')
                        (folder/'commands'/(variant+'-cli-'+mode+'-'+ident+'.stdout')).write_bytes(b'wrong\n')
                        receipt={'argv':matrix.cli_command(original/'binaries'/(variant+'-cli'),original/inp['image'],models,assets,mode,evidence/'cli'/variant/mode/(ident+'.txt')),'exit_code':0}
                        matrix.write_json(folder/'commands'/(variant+'-cli-'+mode+'-'+ident+'.json'),receipt);meta['commands'].append(receipt)
        if kind.startswith('page'):meta.update(Page_API_results=len(selected)*4,PageRecognizer_loads=2,model_session_loads=4)
        else:meta.update(CLI_processes=len(selected)*4,model_session_loads=None)
        if index is not None:
            plan={'kind':kind,'shard':meta['shard'],'global_physical_inputs':4,'bundle_sha256':meta['bundle_sha256']}
            matrix.write_json(folder/'shard-plan.json',plan)
            meta['shard_plan_sha256']=matrix.sha((folder/'shard-plan.json').read_bytes())
        matrix.write_json(folder/'job.json',meta)
    def test_shards_replay_the_same_full_metrics_and_gt_aliases_as_unsharded(self):
        legacy=self.root/'legacy';sharded=self.root/'sharded'
        for kind in ['page30','page50','page100','cli']:
            self.artifact(legacy,kind)
            if kind=='page30':self.artifact(sharded,kind)
            else:
                for index in range(4):self.artifact(sharded,kind,index,4)
        matrix.verify_matrix(self.bundle,legacy,self.root/'legacy-output')
        verify_shards(self.bundle,sharded,self.root/'sharded-output')
        a=json.loads((self.root/'legacy-output/summary.json').read_text());b=json.loads((self.root/'sharded-output/summary.json').read_text())
        for key in ['all-input-pairs.json']:
            self.assertEqual(json.loads((self.root/'legacy-output'/key).read_text()),json.loads((self.root/'sharded-output'/key).read_text()))
        self.assertTrue(b['matrix_gate_pass']);self.assertEqual(b['independently_verified_CSV_rows'],240)
        self.assertEqual(b['successful_job_artifacts'],13)
        self.assertEqual(b['successful_PageRecognizer_loads'],18)
        self.assertEqual(b['successful_Page_model_session_loads'],36)
        self.assertEqual(b['successful_CLI_processes'],16)
        self.assertEqual(a['totals'],b['totals'])
    def test_missing_shard_cannot_enter_scoring_or_claim_full_gate(self):
        artifacts=self.root/'incomplete'
        self.artifact(artifacts,'page30')
        for kind in ['page50','page100','cli']:
            for index in range(3 if kind=='cli' else 4):self.artifact(artifacts,kind,index,4)
        with self.assertRaises(ValueError):verify_shards(self.bundle,artifacts,self.root/'output')
        self.assertFalse((self.root/'output/summary.json').exists())

    def test_missing_pre_nn_plan_is_rejected_even_when_every_result_is_complete(self):
        artifacts=self.root/'missing-plan'
        self.artifact(artifacts,'page30')
        for kind in ['page50','page100','cli']:
            for index in range(4):self.artifact(artifacts,kind,index,4)
        (artifacts/'page50-0/shard-plan.json').unlink()
        with self.assertRaises(ValueError):verify_shards(self.bundle,artifacts,self.root/'output')

if __name__=='__main__':unittest.main()

