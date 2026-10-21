"""Frozen FVC v2 bundle adapter for the unchanged complete-matrix runner."""
import json
from pathlib import Path
import vertical_cli_fallback_full_matrix as runner
from vertical_page_matrix import safe_member,sha,write_json
from verify_additional_real_baseline import _packet
from verify_vertical_cli_fallback_v2 import verify_new72
from vertical_cli_fallback_v2_full_scope import combine_scope
from vertical_cli_fallback_v2_option_runner import require_new72_gate


class FullBundleV2(runner.FullBundle):
    def __init__(self,folder):
        self.folder=Path(folder);self.index=json.loads((self.folder/'bundle.json').read_text())
        if self.index['schema']!=2:raise ValueError('unsupported v2 full-bundle schema')
        assert sha((self.folder/'protocol-before-nn.md').read_bytes())==self.index['protocol_sha256']
        parent=self.folder.parent;previous_folder=parent.parent/'vertical-cli-fallback/full-matrix-v1'
        assert sha((previous_folder/'bundle.json').read_bytes())==self.index['previous_bundle_sha256']
        self.new72_gate=verify_new72(parent);require_new72_gate(self.new72_gate)
        assert self.new72_gate['archive_sha256']==self.index['new72_packet_sha256']
        before_index,self.new_data=_packet(parent,'before-nn-v1.index.json')
        assert before_index['sha256']==self.index['before_packet_sha256']
        before=json.loads(self.new_data['evidence/before-NN-v1.json']);manifest=json.loads(self.new_data['evidence/new-manifest.json'])
        self.original_root=Path(before['frozen_candidate_cli']['path']).parents[2]
        self.previous=runner.FullBundle(previous_folder);self.base=self.previous.base
        images,scores=combine_scope(self.previous.index,manifest)
        assert images==self.index['physical_inputs'] and scores==self.index['scoring_inputs']
        assert self.index['assets']==self.previous.index['assets']
        self.sources={variant:{name:self.new_data[variant+'-source/'+name] for name in expected} for variant,expected in self.index['source_files'].items()}
        assert self.index['source_files']['baseline']==before['baseline_source_files']
        assert self.index['source_files']['candidate']==before['source_files']
        for variant,files in self.sources.items():
            assert all(sha(data)==self.index['source_files'][variant][name] for name,data in files.items())
        assert self.sources['baseline']==self.previous.sources['baseline']
        self.baseline_FVB_bridge=self.previous.baseline_FVB_bridge
        assert all(sha(self.read_image(row))==row['image_sha256'] for row in images)
        assert all(sha(self.read_truth(row))==row['truth_sha256'] for row in scores)
        self.plan=runner.full_plan(images,scores,self.index['shards'])
        assert self.plan==json.loads((self.folder/'plan.json').read_text())
        expected=self.index['expected']
        assert len(images)==expected['physical_inputs']==1311 and len(scores)==expected['scoring_inputs']==1317
        assert len({(r['width'],r['height'],r['rgb_sha256']) for r in images})==expected['unique_RGB']==1310
        assert self.plan['expected_image_results']==expected['image_results']==26220
        assert self.plan['expected_CSV_rows']==expected['CSV_rows']==79020

    def _new_blob(self,path):
        prefix=str(self.original_root)+'/'
        if not path.startswith(prefix+'tmp/vertical-cli-fallback-v2/inputs/'):
            raise ValueError('unexpected v2 control transport path')
        return self.new_data[str(safe_member(path[len(prefix):]))]

    def read_image(self,row):
        return self._new_blob(row['image']) if row['origin']=='fvc_v2_new72' else self.previous.read_image(row)

    def read_truth(self,row):
        return self._new_blob(row['truth']) if row['origin']=='fvc_v2_new72' else self.previous.read_truth(row)


def configure_harness():
    additions=('tools/vertical_cli_fallback_v2_full_matrix.py','tools/vertical_cli_fallback_v2_full_scope.py',
               'tools/vertical_cli_fallback_v2_option_runner.py','tools/verify_vertical_cli_fallback_v2.py',
               'tools/verify_additional_real_baseline.py','tools/additional_real_diagnostic.py',
               'tools/low_stop_evidence_requirements.txt')
    runner.HARNESS_FILES=tuple(dict.fromkeys(runner.HARNESS_FILES+additions))


def main():
    import argparse
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--bundle',type=Path,default=Path(__file__).resolve().parents[1]/'docs/benchmarks/vertical-cli-fallback-v2/full-matrix-v1')
    parser.add_argument('--validate',action='store_true');parser.add_argument('--build',action='store_true')
    parser.add_argument('--kind',choices=runner.KINDS);parser.add_argument('--index',type=int);parser.add_argument('--total',type=int,default=8)
    parser.add_argument('--stage',type=Path);parser.add_argument('--artifacts',type=Path);parser.add_argument('--output',type=Path);parser.add_argument('--models',type=Path,default=Path('models'))
    args=parser.parse_args()
    try:
        bundle=FullBundleV2(args.bundle);configure_harness()
        if args.validate:print(json.dumps({'validated':True,'expected':bundle.index['expected'],'jobs':len(bundle.plan['jobs']),'NN_calls':0,'new72_gate_pass':bundle.new72_gate['matrix_gate_pass']}))
        elif args.build and args.output:runner.build_pair(bundle,args.output)
        elif args.kind and args.output and args.stage and args.index is not None:runner.run_shard(bundle,args.stage,args.kind,args.index,args.total,args.output,args.models)
        elif args.artifacts and args.output and args.stage:runner.compare_all(bundle,args.stage,args.artifacts,args.output)
        else:parser.error('choose --validate, --build, --kind or --artifacts with required paths')
    except Exception as error:
        if args.output:
            args.output.mkdir(parents=True,exist_ok=True);write_json(args.output/'failure.json',{'complete':False,'error':repr(error),'kind':args.kind,'index':args.index,'total':args.total})
        raise


if __name__=='__main__':main()
