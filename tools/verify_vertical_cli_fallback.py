"""Replay the isolated FVC new72 CLI comparison without model inference."""
import csv
import io
import json
from pathlib import Path
import zipfile
from vertical_page_matrix import cli_command,safe_member,sha
from verify_low_stop_evidence import metric_values


def packet(folder,index_name):
    index=json.loads((folder/index_name).read_text());archive=folder/safe_member(index['archive']);raw=archive.read_bytes()
    assert len(raw)==index['bytes'] and sha(raw)==index['sha256']
    with zipfile.ZipFile(archive) as z:
        assert len(z.namelist())==len(set(z.namelist())) and set(z.namelist())==set(index['members'])
        data={}
        for name,expected in index['members'].items():
            safe_member(name);raw=z.read(name);assert len(raw)==expected['bytes'] and sha(raw)==expected['sha256'];data[name]=raw
    return index,data


def verify(folder):
    folder=Path(folder);index,data=packet(folder,'new72-cli-matrix-v1.json');pure_index,pure=packet(folder,'index.json');adapter_index,adapter=packet(folder,'adapter-before-nn-v1.json')
    assert index['input_pure_packet_sha256']==pure_index['sha256'] and index['source_adapter_packet_sha256']==adapter_index['sha256']
    job=json.loads(data['new72/job.json']);summary=json.loads(data['new72/summary.json'])
    assert job['complete'] and job['frozen_materials_after_unchanged'] and job['image_results']==576
    assert job['CI']['conclusion']=='success' and len(job['CI']['jobs'])==5 and all(j['conclusion']=='success' for j in job['CI']['jobs'])
    assert job['internal_forward_calls'] is None and job['model_session_loads'] is None
    assert sha(data['evidence/run_new72_matrix_v1.py'])==job['driver_sha256']
    raw=data['evidence/new-manifest.json'];assert sha(raw)==job['manifest_sha256'];manifest=json.loads(raw);rows=manifest['inputs'];assert len(rows)==72 and len({r['id'] for r in rows})==72
    assert len(job['commands'])==579 and all(r['exit_code']==0 for r in job['commands'])
    prefix='tmp/vertical-cli-fallback/'
    for variant,plan in job['plans'].items():
        frozen=plan['frozen'];assert frozen==json.loads(adapter[prefix+'cli-regression-'+variant+'-frozen.json'])
        with zipfile.ZipFile(io.BytesIO(adapter[prefix+'adapter-final-'+variant+'-source.zip'])) as source:
            for name,digest in frozen['source_files'].items():assert sha(source.read(name))==digest
        name='cli-regression-'+variant+'-build'+('-v2' if variant=='candidate' else '')+'.jsonl'
        events=[json.loads(line) for line in adapter[prefix+name].decode().splitlines() if line.startswith('{')]
        for key in ['cli','test']:
            event=frozen[key]['compiler_artifact'];assert event in events and event['fresh'] is False and 'onnx' in event['features']
    assert job['plans']['baseline']['frozen']['cli']['sha256']!=job['plans']['candidate']['frozen']['cli']['sha256']
    known=json.loads(data['known/receipt.json']);assert sha(data['known/receipt.json'])==job['known_regression_receipt_sha256'] and known['complete']
    assert [r['exit_code'] for r in known['runs']]==[101,0] and known['new_control_gain_count']==0
    assert 'literal CLI body, deim=false' in data['known/baseline.stderr'].decode()
    assert b'1 passed; 0 failed' in data['known/candidate.stdout']
    metadata={tuple(r['argv']):r for r in job['commands']};assert len(metadata)==579
    for n,raw in data.items():
        if n.startswith('new72/commands/') and n.endswith('.json'):
            receipt=json.loads(raw);assert receipt==metadata[tuple(receipt['argv'])]
    predictions={};processes=0
    workspace=Path(pure_index['workspace_root'])
    for row in rows:
        for key in ['image','rgb','truth']:
            member=Path(row[key]).relative_to(workspace).as_posix();assert sha(pure[member])==row[key+'_sha256']
        assert data['new72/metrics/truth/'+row['id']+'.txt']==pure[Path(row['truth']).relative_to(workspace).as_posix()]
    for variant,plan in job['plans'].items():
        for mode in ['deim','bands']:
            for cascade in [True,False]:
                route=mode+'_cascade_'+str(cascade).lower();key=variant+'_'+route;predictions[key]={}
                for row in rows:
                    ident=row['id'];body=data['new72/predictions/'+key+'/'+ident+'.txt'];label=key+'_'+ident
                    assert data['new72/commands/'+label+'.stdout']==body+b'\n'
                    receipt=json.loads(data['new72/commands/'+label+'.json']);argv=receipt['argv'];models=Path(argv[argv.index('--model')+1]).parent
                    expected=cli_command(plan['frozen']['cli']['path'],row['image'],models,job['assets'],mode,Path(argv[argv.index('--output-txt')+1]))+['--enable-cascade',str(cascade).lower()]
                    assert argv==expected;predictions[key][ident]=body.decode();processes+=1
    verified=0;comparisons=[];gains=[];regressions=[]
    for normalization in ['none','width','strict']:
        table=list(csv.DictReader(io.StringIO(data['new72/metrics/'+normalization+'.csv'].decode())))
        assert len(table)==576 and len({(r['system'],r['file']) for r in table})==576
        for metric in table:
            expected=metric_values(data['new72/metrics/truth/'+metric['file']].decode(),data['new72/predictions/'+metric['system']+'/'+metric['file']].decode(),normalization)
            assert tuple(int(metric[k]) for k in ['char_errors','char_total','word_errors','word_total'])==expected;verified+=1
        metrics={(r['system'],r['file']):r for r in table}
        for row in rows:
            for mode in ['deim','bands']:
                for cascade in [True,False]:
                    route=mode+'_cascade_'+str(cascade).lower();before=metrics['baseline_'+route,row['id']+'.txt'];after=metrics['candidate_'+route,row['id']+'.txt']
                    entry={'normalization':normalization,'id':row['id'],'cohort':row['cohort'],'route':route,'before':before,'after':after,'body_unchanged':predictions['baseline_'+route][row['id']]==predictions['candidate_'+route][row['id']]};comparisons.append(entry)
                    if any(int(after[k])>int(before[k]) for k in ['char_errors','word_errors']):regressions.append(entry)
                    if normalization=='none' and row['cohort']=='new_fvc_vertical_text' and int(after['char_errors'])<int(before['char_errors']) and int(after['word_errors'])<=int(before['word_errors']):gains.append(entry)
    for name,expected in [('comparisons',comparisons),('new-body-gains',gains),('regressions',regressions)]:assert json.loads(data['new72/'+name+'.json'])==expected
    assert processes==576 and verified==1728 and len(gains)==30 and not regressions
    assert summary['matrix_gate_pass'] and summary['new_body_raw_gain_pairs']==len(gains)
    return {'complete':True,'image_results':processes,'verified_CSV_rows':verified,'new_body_raw_gain_pairs':len(gains),
            'new_body_physical_images':len({r['id'] for r in gains}),'regression_pairs':len(regressions),'new72_gate_pass':True,'full_gate_pass':False}


if __name__=='__main__':
    import argparse
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--folder',type=Path,default=Path(__file__).resolve().parents[1]/'docs/benchmarks/vertical-cli-fallback')
    args=parser.parse_args();print(json.dumps(verify(args.folder),indent=2))
