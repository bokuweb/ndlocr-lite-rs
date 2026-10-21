"""Authenticate real primary stop diagnostics, including every original Page route."""
from collections import Counter
import io
import json
from pathlib import Path
import struct
import zipfile
from additional_real_stop_probe import unique_primary_inputs,rotate_primary
from verify_additional_real_baseline import _packet
from verify_additional_real_primary_trace import verify as verify_trace
from vertical_page_matrix import sha,safe_member


def verify(folder):
    folder=Path(folder);trace_folder=folder.parent/'real-vertical-additional-primary-trace';verify_trace(trace_folder)
    ti=json.loads((trace_folder/'trace-v1.index.json').read_text());trace={}
    with zipfile.ZipFile(trace_folder/ti['archive']) as z:
        for name,meta in ti['members'].items():
            raw=z.read('objects/'+meta['sha256']);assert sha(raw)==meta['sha256'];trace[name]=raw
    bi,before_data=_packet(folder,'before-nn-v1.index.json');ri,data=_packet(folder,'results-v1.index.json');_,bridge=_packet(folder,'source-bridge-v1.index.json')
    assert bi['primary_trace_packet_sha256']==ti['sha256'] and ri['before_packet_sha256']==bi['sha256']
    before=json.loads(before_data['evidence/before-NN-v1.json']);job=json.loads(data['run/job-v1.json']);refs=json.loads(before_data['evidence/references-v1.json'])
    assert before['NN_calls_at_freeze']==0 and not before['complete'] and before['GT_passed'] is False and before['new_inference_policy'] is False
    assert job['complete'] and job['frozen_files_after_unchanged'] and job['files_sha256']==before['files_sha256']
    assert all(job[k]==v for k,v in before.items() if k!='complete')
    artifact=json.loads(before_data['baseline-evidence/native-baseline-artifact.json']);sources=json.loads(before_data['baseline-evidence/baseline-source-bindings.json'])
    assert artifact['artifact']['fresh'] is False and 'onnx' in artifact['artifact']['features']
    events=[json.loads(line) for line in before_data['baseline-evidence/native-baseline-build.jsonl'].decode().splitlines() if line.startswith('{')];assert artifact['artifact'] in events
    for name,digest in sources.items():assert sha(before_data['baseline-source/'+name])==digest
    root=Path(before['commands'][0]['argv'][0]).parents[2];containers=json.loads(trace['transport/zip-members.json']);assets=json.loads(trace['run/origin-v1.json'])['models']
    for path,digest in before['files_sha256'].items():
        name=str(Path(path).relative_to(root));raw=None
        if name.startswith('tmp/vertical-unique-insertion/baseline-source/'):raw=before_data['baseline-source/'+name.split('/baseline-source/',1)[1]]
        elif name=='tmp/vertical-unique-insertion/native-baseline-binary':assert digest==artifact['binary_sha256'];continue
        elif name.startswith('tmp/vertical-unique-insertion/'):raw=before_data['baseline-evidence/'+Path(name).name]
        elif name.startswith('tmp/real-vertical-additional-primary-trace/'):
            key='run/'+name.split('/real-vertical-additional-primary-trace/',1)[1]
            if key.endswith('.zip'):assert containers[key]['original_container_sha256']==digest;continue
            raw=trace[key]
        elif name.startswith('tmp/real-vertical-additional-stop-probe/'):raw=before_data['evidence/'+name.split('/real-vertical-additional-stop-probe/',1)[1]]
        elif name.startswith('models/'):assert {Path(n).name:h for n,h in assets.items()}[Path(name).name]==digest;continue
        else:raw=bridge[name] if name in bridge else before_data[name]
        assert sha(raw)==digest,name
    for name,digest in before['source_bridge'].items():
        a=before_data['baseline-source/'+name].decode();b=bridge[name].decode()
        if name.endswith('cached.rs'):
            start='    pub fn recognize_rgb_u8_candidates_with_orientation(';extract=lambda text:text[text.index(start):text.index('\n    }',text.index(start))+6]
        else:
            start='pub fn predict_text_from_flat_logits_with_confidence(';end='pub fn load_charset_from_yaml_str(';extract=lambda text:text[text.index(start):text.index(end,text.index(start))]
        assert extract(a)==extract(b) and sha(extract(a).encode())==digest
    rows=[]
    for ref in refs['all_original_routes']:
        name='run/'+ref['trace_json'].split('/real-vertical-additional-primary-trace/',1)[1];record=json.loads(trace[name]);primary,=[r for r in record['events'] if r['kind']=='Page_primary_crop'];raw,=[r for r in record['events'] if r['kind']=='raw_before_H_O']
        key='run/'+ref['trace_zip'].split('/real-vertical-additional-primary-trace/',1)[1]+'!/'
        rgb=trace[key+ref['primary_member']];assert rgb==trace[key+ref['raw_member']]
        assert raw['data']['orientation']=='Auto' and primary['image']['height']*5>primary['image']['width']*4
        assert ref['origin']==record['model']+'_'+record['option']+'_'+record['id']
        rows.append({'origin':ref['origin'],'model':record['model'],'width':primary['image']['width'],'height':primary['image']['height'],'rgb':rgb,'raw':raw['data']['text'],'mean_bits':raw['data']['mean_bits']})
    unique=unique_primary_inputs(rows);assert len(rows)==36 and len(unique)==18
    with zipfile.ZipFile(io.BytesIO(before_data['evidence/rotated-primary-v1.zip'])) as z:
        for row,ref in zip(unique,refs['unique_inputs'],strict=True):
            rotated,w,h=rotate_primary(row['rgb'],row['width'],row['height'])
            assert ref=={k:v for k,v in row.items() if k!='rgb'}|{'rotated_width':w,'rotated_height':h,'rotated_rgb_sha256':sha(rotated),'member':row['id']+'.rgb'}
            assert z.read(ref['member'])==rotated
            for y in range(row['height']):
                for x in range(row['width']):
                    a=(y*row['width']+x)*3;b=((row['width']-1-x)*row['height']+y)*3;assert rotated[b:b+3]==row['rgb'][a:a+3]
    bits=lambda value:struct.unpack('!I',struct.pack('!f',value))[0];f32=lambda value:struct.unpack('!f',struct.pack('!f',value))[0]
    counts=Counter();matched=0;mismatches=0;actual_count=0
    for command,executed in zip(before['commands'],job['executed_commands'],strict=True):
        model=command['model'];assert all(executed[k]==v for k,v in command.items()) and executed['exit_code']==0 and executed['frozen_files_after_unchanged']
        assert b'1 passed; 0 failed;' in data[f'run/run-{model}-v1.stdout']
        manifest=json.loads(before_data[f'evidence/NN-{model}-v1.json']);selected=[r for r in refs['unique_inputs'] if r['model']==model]
        assert manifest['expected_items']==len(selected)==6 and manifest['input_dimensions']==[{'30':256,'50':384,'100':768}[model],24]
        assert manifest['items']==[{'id':r['id'],'archive':str(root/'tmp/real-vertical-additional-stop-probe/rotated-primary-v1.zip'),'member':r['member'],'sha256':r['rotated_rgb_sha256'],'width':r['rotated_width'],'height':r['rotated_height']} for r in selected]
        assert Path(manifest['model']['path']).name in {Path(n).name for n in assets} and manifest['model']['sha256']==assets[manifest['model']['path']]
        loaded=json.loads(data[f'run/results-{model}-v1/model-loaded.json']);assert loaded['loaded_session'] and loaded['model']==manifest['model']
        results=json.loads(data[f'run/results-{model}-v1/results.json']);assert [r['id'] for r in results]==[r['id'] for r in selected]
        with zipfile.ZipFile(io.BytesIO(data[f'run/results-{model}-v1/actual-rgb-inputs.zip'])) as z:
            for row,ref in zip(results,selected,strict=True):
                assert sha(z.read(row['id']+'.rgb'))==row['expected_rgb_sha256']==ref['rotated_rgb_sha256'] and row['orientation']=='Horizontal' and row['top_k']==row['api_calls']==1
                same=(row['text'],row['mean_bits'])==(ref['raw'],ref['mean_bits']);matched+=len(ref['origins']) if same else 0;mismatches+=not same;actual_count+=1
                emitted=bytearray();total=0.0;count=0
                for ordinal,step in enumerate(row['steps']):
                    candidate,=step['candidates'];p=candidate['probability'];assert step['timestep']==ordinal and bits(p)==candidate['probability_bits'] and 0<=p<=1
                    if step['text_range'] is not None:
                        encoded=candidate['character'].encode();assert candidate['token_id']>0 and step['text_range']==[len(emitted),len(emitted)+len(encoded)];emitted.extend(encoded);total=f32(total+p);count+=1
                assert emitted.decode()==row['text'] and bits(f32(total/count if count else 0))==row['mean_bits']
                last=row['steps'][-1];winner=last['candidates'][0];reason=row['stop_reason'];counts[reason]+=1
                if reason=='EndOfSequence':assert winner['token_id']==0 and winner['character'] is None and last['text_range'] is None
                elif reason=='LowConfidence':assert winner['token_id']>0 and winner['probability']<f32(.35) and count>=2 and last['text_range'] is None
                else:assert reason=='TimestepsExhausted' and last['text_range'] is not None
    assert actual_count==before['unique_raw_API_calls_planned']==ri['raw_API_calls']==18 and len(job['executed_commands'])==3
    return {'raw_API_calls':actual_count,'original_routes_matched':matched,'raw_or_confidence_mismatches':mismatches,'stop_counts':dict(counts),'model_session_loads':3,'new_model_calls_in_replay':0,'new_accuracy_gain':False}


if __name__=='__main__':
    print(json.dumps(verify(Path(__file__).resolve().parents[1]/'docs/benchmarks/real-vertical-additional-stop-probe'),indent=2))
