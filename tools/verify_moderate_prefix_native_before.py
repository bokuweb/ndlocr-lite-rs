"""Authenticate frozen R native sources, inputs and planned commands without NN."""
import io
import json
from pathlib import Path
import zipfile
from moderate_prefix_matrix_scope import matrix_scope
from verify_additional_real_baseline import _packet
from verify_moderate_prefix_packet import verify as verify_controls
from verify_vertical_cli_fallback_v2_root_completed import baseline_sources
from vertical_page_matrix import sha


def verify(folder):
    folder=Path(folder);verify_controls(folder)
    index,data=_packet(folder,'before-native-v1.index.json');ni,controls=_packet(folder,'nn0-v1.index.json');root_folder=folder.parent/'vertical-cli-fallback-v2';ri,root_data=_packet(root_folder/'root-bridge-v1','completed-local-v2.index.json')
    assert index['NN0_controls_packet_sha256']==ni['sha256'] and index['Root_completed_packet_sha256']==ri['sha256'] and index['native_bytes_in_packet'] is False
    _,before_root,_,_=baseline_sources(root_folder);root_frozen=json.loads(root_data['evidence/root-candidate-frozen-v3.json']);root_source=dict(before_root)
    for name,raw in root_data.items():
        if name.startswith('source/') and name[7:] in root_frozen['source_files']:root_source[name[7:]]=raw
    assert {n:sha(b) for n,b in root_source.items()}==root_frozen['source_files'] and len(root_source)==852
    pair=json.loads(data['evidence/native-pair-frozen-v1.json']);candidate=root_source|{n[len('candidate-source/'):]:b for n,b in data.items() if n.startswith('candidate-source/')};sources={'baseline':root_source,'candidate':candidate}
    assert len(candidate)==855 and sorted(n for n,b in candidate.items() if root_source.get(n)!=b)==sorted(index['candidate_changed_files']) and len(index['candidate_changed_files'])==6
    binaries={}
    for variant,source in sources.items():
        meta=pair[variant];assert {n:sha(b) for n,b in source.items()}==meta['source_files']
        raw=data[f'evidence/native-{variant}-build-v1.jsonl'];assert sha(raw)==meta['build_stdout_sha256']
        events=[json.loads(line) for line in raw.decode().splitlines() if line.startswith('{')];artifact=meta['compiler_artifact'];assert artifact in events and artifact['fresh'] is False and 'onnx' in artifact['features'] and artifact['profile']['test']
        binaries[meta['path']]=meta['sha256']
    assert pair['baseline']['sha256']!=pair['candidate']['sha256']
    pre=json.loads(data['evidence/native-before-NN-v1.json']);rows=json.loads(data['evidence/new-manifest-v2.json'])['inputs'];assert data['evidence/new-manifest-v2.json']==controls['evidence/new-manifest-v2.json']
    scope=matrix_scope([r['id'] for r in rows]);assert pre['scope']==scope and pre['NN_calls_at_freeze']==0 and not pre['complete'] and not pre['commands'] and pre['GT_or_expected_predictions_supplied_to_recognizer'] is False
    root=Path(pair['baseline']['path']).parents[2];project=root/'tmp/vertical-moderate-prefix';assets=json.loads((folder.parent/'vertical-page-fallback/bundle.json').read_text())['assets'];asset_names={a['name']:a['sha256'] for a in assets.values()}
    manifest=json.loads(data['evidence/native-manifest-v1.json']);expected_models=[{'label':label,'path':str(root/'models'/assets[label]['name']),'sha256':assets[label]['sha256']} for label in scope['models']]
    assert manifest=={'models':expected_models,'expected_per_model':72,'cases':[{'id':r['id'],'label':m['label'],'rgb':r['rgb'],'width':r['width'],'height':r['height']} for m in expected_models for r in rows]}
    inputs={}
    with zipfile.ZipFile(io.BytesIO(controls['evidence/geometry-original-RGB-v1.zip'])) as z:
        for row in rows:
            for key,raw in [('rgb',z.read(row['id']+'.rgb')),('image',controls['inputs/'+row['id']+'.png']),('truth',controls['inputs/'+row['id']+'.txt'])]:assert sha(raw)==row[key+'_sha256'];inputs[row[key]]=raw
    for absolute,digest in pre['files_sha256'].items():
        if absolute in inputs:assert sha(inputs[absolute])==digest;continue
        if absolute in binaries:assert binaries[absolute]==digest;continue
        path=Path(absolute);relative=str(path.relative_to(root));raw=None
        for variant in sources:
            prefix='tmp/vertical-moderate-prefix/source-'+variant+'/'
            if relative.startswith(prefix):raw=sources[variant][relative[len(prefix):]];break
        if raw is not None:assert sha(raw)==digest;continue
        if relative.startswith('models/'):assert asset_names[path.name]==digest;continue
        if relative.startswith('tmp/vertical-moderate-prefix/'):raw=data['evidence/'+path.name]
        else:raw=data[relative]
        assert sha(raw)==digest,relative
    commands=[]
    for variant in scope['variants']:
        for model in expected_models:
            label=variant+'-'+model['label'];commands.append({'label':label,'variant':variant,'model':model['label'],'argv':[pair[variant]['path'],'--ignored','--exact','infer::vertical_capacity::native::matrix::capture_all_uncertain_native_routes_without_gt','--nocapture','--test-threads=1'],'env':{'NDLOCR_UNCERTAIN_MATRIX_MANIFEST':str(project/'native-manifest-v1.json'),'NDLOCR_UNCERTAIN_MATRIX_MODEL':model['label'],'NDLOCR_UNCERTAIN_MATRIX_OUTPUT':str(project/'native-matrix-v1'/label),'NDLOCR_TEST_CHARSET':str(root/'models'/assets['charset']['name']),'NDLOCR_INTRA_THREADS':'1','NDLOCR_PARSEQ_PARALLELISM':'1'}})
    assert pre['commands_planned']==commands
    collector='tests/support/vertical_uncertain_matrix.rs';assert root_source[collector]==candidate[collector] and b'assert!([72, 414].contains(&count));' in root_source[collector]
    assert b'487 passed; 0 failed; 114 ignored;' in data['evidence/native-candidate-nonignored-v1.stdout']
    assert index['NN_calls_in_replay']==0
    return {'baseline_source_files':852,'candidate_source_files':855,'inputs':len(rows),'planned_image_results':scope['image_results'],'planned_commands':len(commands),'NN_calls_in_replay':0}


if __name__=='__main__':
    print(json.dumps(verify(Path(__file__).resolve().parents[1]/'docs/benchmarks/vertical-moderate-prefix'),indent=2))
