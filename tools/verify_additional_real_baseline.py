"""Replay the frozen real-six baseline; no model, font, or native binary needed."""
import csv
import io
import json
from pathlib import Path
import zipfile
from additional_real_diagnostic import text_diagnosis
from vertical_page_matrix import safe_member, sha
from verify_low_stop_evidence import metric_values


def _packet(folder, index_name):
    index=json.loads((folder/index_name).read_text())
    archive=folder/safe_member(index['archive'])
    assert archive.stat().st_size==index['bytes'] and sha(archive.read_bytes())==index['sha256']
    data={}
    with zipfile.ZipFile(archive) as z:
        assert len(z.namelist())==len(set(z.namelist())) and set(z.namelist())==set(index['members'])
        for name,expected in index['members'].items():
            safe_member(name)
            raw=z.read(name)
            assert len(raw)==expected['bytes'] and sha(raw)==expected['sha256'],name
            data[name]=raw
    return index,data


def verify(folder):
    folder=Path(folder)
    index,data=_packet(folder,'FVB-baseline-real6-v1.index.json')
    _,controls=_packet(folder,'index.json')
    master=json.loads(controls['tmp/real-vertical-additional-controls/additional-controls-manifest-v1.json'])
    job=json.loads(data['run/job.json']);summary=json.loads(data['run/summary.json'])
    assert len(master['inputs'])==6 and len(master['scoring_inputs'])==11 and master['GT_frozen']
    assert job['complete'] and job['frozen_materials_after_unchanged'] and not job['adoption_comparison']
    assert summary['image_results']==48 and summary['verified_CSV_rows']==264
    assert job['baseline_CI']['headSha']==index['baseline_head'] and job['baseline_CI']['conclusion']=='success'
    assert len(job['baseline_CI']['jobs'])==5 and all(j['conclusion']=='success' for j in job['baseline_CI']['jobs'])
    frozen=job['baseline']
    for name,digest in frozen['source_files'].items():assert sha(data['baseline-source/'+name])==digest,name
    assert sha(data['evidence/run_FVB_baseline.py'])==job['driver_sha256']
    assert sha(data['run/NN-images-only.json'])==job['manifest_sha256']
    images=json.loads(data['run/NN-images-only.json'])['inputs']
    assert {r['id'] for r in images}=={r['id'] for r in master['inputs']}
    assert all(set(r)=={'id','image'} for r in images)
    assert len(job['commands'])==18 and all(r['exit_code']==0 for r in job['commands'])
    receipts=[json.loads(raw) for name,raw in data.items() if name.startswith('run/commands/') and name.endswith('.json')]
    assert len(receipts)==18
    from collections import Counter
    assert Counter(tuple(r['argv']) for r in receipts)==Counter(tuple(r['argv']) for r in job['commands'])
    assert all(r['exit_code']==0 for r in receipts)
    build=data['evidence/root-baseline-build.jsonl']
    assert sha(build)==frozen['build_stdout_sha256']
    events=[json.loads(line) for line in build.decode().splitlines() if line.startswith('{')]
    for binary in frozen['binaries'].values():
        event=binary['compiler_artifact']
        assert event in events and event['fresh'] is False and 'onnx' in event['features'] and not event['profile']['test']
    page_receipts=[r for r in receipts if r['argv'][0]==frozen['binaries']['page']['path']]
    cli_receipts=[r for r in receipts if r['argv'][0]==frozen['binaries']['cli']['path']]
    assert len(page_receipts)==3 and len(cli_receipts)==12
    for receipt in page_receipts:
        argv=receipt['argv'];label=argv[argv.index('--label')+1];model=label.removeprefix('runtime')
        assert model in {'30','50','100'}
        assert Path(argv[argv.index('--parseq-model')+1]).name==job['assets'][model]['name']
    for receipt in page_receipts+cli_receipts:
        argv=receipt['argv']
        for option,asset in [('--det-model','deim'),('--charset','charset')]:
            assert Path(argv[argv.index(option)+1]).name==job['assets'][asset]['name']
    for receipt in cli_receipts:
        argv=receipt['argv']
        for option,asset in [('--model','100'),('--model30','30'),('--model50','50')]:
            assert Path(argv[argv.index(option)+1]).name==job['assets'][asset]['name']
    counts=('char_errors','char_total','word_errors','word_total');verified=0;routes=set()
    for mode in ['none','width','strict']:
        rows=list(csv.DictReader(io.StringIO(data['run/metrics/'+mode+'.csv'].decode())))
        assert len(rows)==88 and len({(r['system'],r['file']) for r in rows})==88
        for row in rows:
            truth=data['run/metrics/truth/'+row['file']].decode()
            prediction=data['run/metrics/'+row['system']+'/'+row['file']].decode()
            expected=metric_values(truth,prediction,mode)
            assert tuple(int(row[k]) for k in counts)==expected
            routes.add(row['system']);verified+=1
    assert routes=={'30_eval','30_default','50_eval','50_default','100_eval','100_default','cli_deim','cli_bands'}
    physical={r['id'] for r in master['inputs']}
    for route in sorted(routes):
        if route.startswith('cli_'):
            predictions={ident:data['run/cli/'+route[4:]+'/'+ident+'.txt'].decode() for ident in physical}
            for ident,text in predictions.items():
                label='cli-'+route[4:]+'-'+ident
                assert data['run/commands/'+label+'.stdout']==(text+'\n').encode()
        else:
            model,setting=route.split('_');rows=json.loads(data[f'run/page/{model}/runtime{model}_{setting}/results.json'])
            assert {r['id'] for r in rows}==physical and len(rows)==6
            predictions={r['id']:r['text'] for r in rows}
        for alias in master['scoring_inputs']:
            truth=data['run/metrics/truth/'+alias['id']+'.txt']
            assert truth==controls[alias['truth']] and sha(truth)==alias['truth_sha256']
            assert data['run/metrics/'+route+'/'+alias['id']+'.txt'].decode()==predictions[alias['physical_id']]
    diagnosis=json.loads(data['run/text-diagnosis-v1.json']);entries=diagnosis['entries']
    assert len(entries)==88 and len({(r['route'],r['scoring_id']) for r in entries})==88
    for entry in entries:
        truth=data['run/metrics/truth/'+entry['scoring_id']+'.txt'].decode()
        prediction=data['run/metrics/'+entry['route']+'/'+entry['scoring_id']+'.txt'].decode()
        assert entry['truth']==truth and entry['prediction']==prediction
        for key,value in text_diagnosis(truth,prediction).items():assert entry[key]==value
    return {'complete':True,'archive_sha256':index['sha256'],'image_results':48,'verified_CSV_rows':verified,'verified_raw_diagnoses':len(entries),'adoption_comparison':False}


if __name__=='__main__':
    import argparse
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--folder',type=Path,default=Path(__file__).resolve().parents[1]/'docs/benchmarks/real-vertical-additional-controls')
    args=parser.parse_args()
    print(json.dumps(verify(args.folder),indent=2))
