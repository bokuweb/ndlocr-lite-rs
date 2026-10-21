"""Authenticate and independently replay the completed, known-input CLI trace."""
import csv
import io
import json
from pathlib import Path
import zipfile
from cli_cascade_trace import trace_command
from vertical_page_matrix import Bundle,check_build_artifacts,safe_member,sha
from verify_low_stop_evidence import metric_values


def verify(folder):
    from PIL import Image
    folder=Path(folder);bundle=Bundle(folder)
    index=json.loads((folder/'complete-linux-trace-v1.json').read_text());archive=folder/safe_member(index['archive'])
    assert archive.stat().st_size==index['bytes'] and sha(archive.read_bytes())==index['sha256']
    with zipfile.ZipFile(archive) as z:
        assert len(z.namelist())==len(set(z.namelist())) and set(z.namelist())==set(index['members'])
        data={}
        for n,h in index['members'].items():
            safe_member(n);raw=z.read(n);assert len(raw)==h['bytes'] and sha(raw)==h['sha256'];data[n]=raw
    job=json.loads(data['job.json'])
    assert job['complete'] and job['frozen_materials_after_unchanged'] and job['all_normal_trace_outputs_equal']
    assert job['bundle_sha256']==sha((folder/'bundle.json').read_bytes())
    assert job['internal_forward_calls'] is None and job['model_session_loads'] is None
    assert job['observer_additional_NN_requests']==0 and not job['adoption_comparison']
    assert job['runtime_environment']=={'NDLOCR_INTRA_THREADS':'1','NDLOCR_PARSEQ_PARALLELISM':'1'}
    for variant,build in job['builds'].items():
        raw=data['commands/'+variant+'-build.stdout'];assert sha(raw)==build['build_stdout_sha256']
        events=[json.loads(line) for line in raw.decode().splitlines() if line.strip()]
        assert check_build_artifacts(events,build['source_manifest'])==build['artifacts']
        assert build['source_files']==bundle.index['source_files'][variant]
    assert job['builds']['baseline']['binaries']['cli']['sha256']!=job['builds']['candidate']['binaries']['cli']['sha256']
    assert b'4 passed; 0 failed' in data['commands/observer-pure-tests.stdout']
    work=Path(job['builds']['baseline']['source_manifest']).parent.parent;evidence=work.parent/'evidence'
    metadata_receipts={tuple(r['argv']):r for r in job['commands']};assert len(metadata_receipts)==24
    for n,raw in data.items():
        if n.startswith('commands/') and n.endswith('.json'):
            receipt=json.loads(raw);assert receipt['exit_code']==0 and receipt==metadata_receipts[tuple(receipt['argv'])]
    processes=0;crop_count=0;batch_count=0;initial100=0;promotions=0
    for config in ['default','model30','model50']:
        for row in bundle.index['physical_inputs']:
            ident=row['id'];image=Image.open(io.BytesIO(bundle.read('data.zip',row['image']))).convert('RGB')
            for variant in ['baseline','candidate']:
                key=variant+'_'+config;label=key+'_'+ident;body=data['predictions/'+key+'/'+ident+'.txt']
                assert data['commands/'+label+'.stdout']==body+b'\n'
                argv=json.loads(data['commands/'+label+'.json'])['argv'];models=Path(argv[argv.index('--charset')+1]).parent
                expected=trace_command(job['builds'][variant]['binaries']['cli']['path'],work/row['image'],models,bundle.index['assets'],config,evidence/'predictions'/key/(ident+'.txt'),work/'not-provided.onnx')
                assert argv==expected
                processes+=1
            for suffix in ['stdout','stderr']:
                assert data['commands/baseline_'+config+'_'+ident+'.'+suffix]==data['commands/candidate_'+config+'_'+ident+'.'+suffix]
            prefix='trace/'+config+'/'+ident+'/'
            events=[json.loads(raw) for n,raw in sorted(data.items()) if n.startswith(prefix) and n.endswith('.json')]
            assert events;phases=[];snapshots={}
            for event in events:
                phase=event['phase'];phases.append(phase)
                assert [r['index'] for r in event['results']]==[i['index'] for i in event['items']]
                if event['kind']=='batch' and event['items']:
                    batch_count+=1
                    assert event['selected_pool_input']==({'default':[768,24],'model30':[256,24],'model50':[384,24]}[config])
                    if config=='default':
                        initial100+=phase=='initial100';promotions+=phase.startswith('retry')
                if event['kind']=='snapshot':snapshots[phase]=event['results']
                for item in event['items']:
                    x0,y0,x1,y1=item['bbox'];crop=item['crop']
                    assert 0<=x0<x1<=image.width and 0<=y0<y1<=image.height
                    assert [crop['width'],crop['height']]==[x1-x0,y1-y0]
                    assert data[prefix+str(safe_member(crop['file']))]==image.crop((x0,y0,x1,y1)).tobytes();crop_count+=1
            assert ('cascade-off' in phases)==(config!='default')
            assert snapshots['initial-final']==snapshots['page-batch-final']
    verified=0
    for mode in ['none','width','strict']:
        rows=list(csv.DictReader(io.StringIO(data['metrics/'+mode+'.csv'].decode())))
        assert len(rows)==18 and len({(r['system'],r['file']) for r in rows})==18
        for row in rows:
            truth=data['metrics/truth/'+row['file']]
            ident=row['file'].removesuffix('.txt');score=next(r for r in bundle.index['scoring_inputs'] if r['id']==ident)
            assert truth==bundle.read('data.zip',score['truth'])
            expected=metric_values(truth.decode(),data['predictions/'+row['system']+'/'+row['file']].decode(),mode)
            assert tuple(int(row[k]) for k in ['char_errors','char_total','word_errors','word_total'])==expected;verified+=1
    assert processes==18 and verified==54 and crop_count==27 and batch_count==9 and initial100==3 and promotions==0
    return {'complete':True,'CLI_processes':processes,'verified_CSV_rows':verified,'verified_crop_instances':crop_count,
            'observed_nonempty_pool_batches':batch_count,'default_nonempty_initial100':initial100,'default_nonempty_promotions':promotions,'adoption_comparison':False}


if __name__=='__main__':
    import argparse
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--folder',type=Path,default=Path(__file__).resolve().parents[1]/'docs/benchmarks/vertical-cli-cascade-trace')
    args=parser.parse_args();print(json.dumps(verify(args.folder),indent=2))
