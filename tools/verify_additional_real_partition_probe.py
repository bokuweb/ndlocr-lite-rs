"""Replay actual primary partitions without changing recognition policy."""
from collections import Counter
import csv
import io
import json
from pathlib import Path
import struct
import zipfile
from verify_additional_real_baseline import _packet
from verify_additional_real_stop_probe import verify as verify_stop,verify_decode_steps
from verify_low_stop_evidence import metric_values
from vertical_page_matrix import sha


def is_subsequence(raw,text):
    remaining=iter(text)
    return all(any(character==wanted for character in remaining) for wanted in raw)


def verify(folder):
    folder=Path(folder);repo=Path(__file__).resolve().parents[1];verify_stop(folder.parent/'real-vertical-additional-stop-probe')
    bi,before_data=_packet(folder,'before-nn-v1.index.json');ri,data=_packet(folder,'results-v1.index.json')
    vi,v2=_packet(folder.parent/'vertical-cli-fallback-v2','before-nn-v1.index.json');si,stop=_packet(folder.parent/'real-vertical-additional-stop-probe','before-nn-v1.index.json')
    assert ri['before_packet_sha256']==bi['sha256'] and bi['source_reference_packet_SHA256']=={'FVCv2':vi['sha256'],'stop':si['sha256']}
    bridge=json.loads(before_data['evidence/source-bridge-v1.json']);frozen=json.loads(before_data['evidence/geometry-binary-frozen-v1.json']);root=Path(frozen['binary_path']).parents[2]
    assert bridge['packet_SHA256']==bi['source_reference_packet_SHA256'] and set(bridge['source_files'])==set(frozen['source_files'])
    root_sources={}
    for name,meta in bridge['source_files'].items():
        raw={'FVCv2':v2,'stop':stop,'this':before_data}[meta['packet']][meta['member']];assert sha(raw)==meta['sha256']==frozen['source_files'][name];root_sources[name]=raw
    artifact=frozen['compiler_artifact'];events=[json.loads(line) for line in before_data['TDD/partition-green-v2.jsonl'].decode().splitlines() if line.startswith('{')]
    assert artifact in events and artifact['fresh'] is False and 'onnx' not in artifact['features']
    assert sha(before_data['TDD/partition-green-v2.jsonl'])==frozen['build_stdout_sha256']
    assert b'2 passed; 0 failed; 1 ignored;' in before_data['TDD/partition-green-v2.jsonl'] and b'442 passed; 0 failed; 39 ignored;' in before_data['evidence/default-full-v1.stdout']
    old=json.loads(stop['baseline-evidence/native-baseline-artifact.json']);assets=json.loads((repo/'docs/benchmarks/vertical-page-fallback/bundle.json').read_text())['assets']
    def check_bindings(bindings):
        for path,digest in bindings.items():
            name=str(Path(path).relative_to(root));raw=None
            if name in root_sources:raw=root_sources[name]
            elif name.startswith('tmp/vertical-unique-insertion/baseline-source/'):raw=stop['baseline-source/'+name.split('/baseline-source/',1)[1]]
            elif name=='tmp/vertical-unique-insertion/native-baseline-binary':assert digest==old['binary_sha256'];continue
            elif name.startswith('tmp/vertical-unique-insertion/'):raw=stop['baseline-evidence/'+Path(name).name]
            elif name=='tmp/real-vertical-primary-partition-probe/geometry-binary':assert digest==frozen['binary_sha256'];continue
            elif name.startswith('tmp/real-vertical-primary-partition-probe/'):raw=before_data['evidence/'+name.split('/real-vertical-primary-partition-probe/',1)[1]]
            elif name.startswith('tmp/real-vertical-additional-stop-probe/'):raw=stop['evidence/'+name.split('/real-vertical-additional-stop-probe/',1)[1]]
            elif name.startswith('models/'):assert {r['name']:r['sha256'] for r in assets.values()}[Path(name).name]==digest;continue
            else:raw=before_data[name]
            assert sha(raw)==digest,name
    geometry=json.loads(before_data['evidence/geometry-command-v1.json']);geometry_before=json.loads(before_data['evidence/geometry-before-v1.json']);pre=json.loads(before_data['evidence/before-part-NN-v1.json']);job=json.loads(data['run/part-job-v1.json'])
    check_bindings(geometry['files_sha256']);check_bindings(pre['files_sha256'])
    assert geometry['complete'] and geometry['exit_code']==0 and geometry['NN_calls']==geometry['model_loads']==0 and geometry['frozen_files_after_unchanged']
    assert geometry['argv']==[frozen['binary_path'],'--ignored','--exact','infer::vertical_capacity::primary_partition_geometry::record_all_primary_partitions_without_nn','--nocapture','--test-threads=1']
    assert geometry['env']=={'NDLOCR_PRIMARY_PARTITION_MANIFEST':str(root/'tmp/real-vertical-primary-partition-probe/geometry-manifest-v1.json'),'NDLOCR_PRIMARY_PARTITION_OUTPUT':str(root/'tmp/real-vertical-primary-partition-probe/geometry-v1')}
    assert geometry['files_sha256']==geometry_before['files_sha256'] and not geometry_before['complete']
    assert pre['NN_calls_at_freeze']==0 and pre['GT_or_raw_expectations_passed'] is False and pre['new_inference_policy'] is False and not pre['complete']
    assert job['complete'] and job['frozen_files_after_unchanged'] and all(job[k]==v for k,v in pre.items() if k!='complete')
    refs=json.loads(before_data['evidence/part-references-v1.json']);stop_refs=json.loads(stop['evidence/references-v1.json'])
    assert refs['all_primary_inputs']==stop_refs['unique_inputs'] and len(refs['all_primary_inputs'])==18
    parents={r['id']:r for r in refs['all_primary_inputs']};geom=json.loads(before_data['evidence/geometry-v1/records.json']);manifest=json.loads(before_data['evidence/geometry-manifest-v1.json'])
    assert [r['id'] for r in geom]==[r['id'] for r in manifest['items']]==list(parents)
    parts=[];pixels=0
    with zipfile.ZipFile(io.BytesIO(before_data['evidence/original-primary-v1.zip'])) as original,zipfile.ZipFile(io.BytesIO(before_data['evidence/geometry-v1/parts.zip'])) as archive,zipfile.ZipFile(io.BytesIO(stop['evidence/rotated-primary-v1.zip'])) as rotated:
        for row,item in zip(geom,manifest['items'],strict=True):
            parent=parents[row['id']];width,height=parent['width'],parent['height'];rgb=original.read(parent['rgb_sha256']+'.rgb')
            assert sha(rgb)==parent['rgb_sha256'] and len(rgb)==width*height*3 and row['error'] is None and len(row['routes'])==2
            assert item=={'id':parent['id'],'archive':str(root/'tmp/real-vertical-primary-partition-probe/original-primary-v1.zip'),'member':parent['rgb_sha256']+'.rgb','width':width,'height':height,'input':[{'30':256,'50':384,'100':768}[parent['model']],24]}
            transformed=rotated.read(parent['member'])
            for route,chunks in enumerate(row['routes']):
                for index,part in enumerate(chunks):
                    raw=archive.read(part['file']);left,right=part['range'];assert 0<=left<right<=height and (part['width'],part['height'])==(right-left,width) and not part['space_before'] and len(raw)==(right-left)*width*3
                    for y in range(width):
                        for x in range(right-left):
                            a=(y*(right-left)+x)*3;b=((left+x)*width+(width-1-y))*3;c=(y*height+left+x)*3
                            assert raw[a:a+3]==rgb[b:b+3]==transformed[c:c+3];pixels+=1
                    ident=sha(f'{parent["model"]}:{row["id"]}:{route}:{index}'.encode())
                    parts.append({'id':ident,'model':parent['model'],'parent_id':row['id'],'origins':parent['origins'],'route':route,'part_index':index,'range':part['range'],'member':part['file'],'width':part['width'],'height':part['height'],'rgb_sha256':sha(raw)})
        assert parts==refs['parts'] and len(parts)==117 and pixels==pre['independent_crop_pixels']
        unique={(r['model'],r['width'],r['height'],r['rgb_sha256']) for r in parts}
        assert len(unique)==pre['unique_model_RGB_inputs'] and len(parts)-len(unique)==pre['repeated_part_calls']
        values={};stops=Counter()
        for command,executed in zip(pre['commands'],job['executed_commands'],strict=True):
            model=command['model'];selected=[r for r in parts if r['model']==model];nn=json.loads(before_data[f'evidence/NN-{model}-v1.json'])
            assert nn['items']==[{'id':r['id'],'archive':str(root/'tmp/real-vertical-primary-partition-probe/geometry-v1/parts.zip'),'member':r['member'],'sha256':r['rgb_sha256'],'width':r['width'],'height':r['height']} for r in selected] and nn['expected_items']==len(selected)==39
            assert nn['model']=={'label':model,'path':str(root/'models'/assets[model]['name']),'sha256':assets[model]['sha256']} and nn['input_dimensions']==[{'30':256,'50':384,'100':768}[model],24]
            assert command['argv']==[str(root/'tmp/vertical-unique-insertion/native-baseline-binary'),'--ignored','--exact','infer::vertical_capacity::prefix::capture::capture_all_frozen_verified_chunks_with_raw_candidates','--nocapture','--test-threads=1']
            assert command['env']=={'NDLOCR_VERIFIED_NN_MANIFEST':str(root/f'tmp/real-vertical-primary-partition-probe/NN-{model}-v1.json'),'NDLOCR_VERIFIED_NN_OUTPUT':str(root/f'tmp/real-vertical-primary-partition-probe/part-results-{model}-v1'),'NDLOCR_TEST_CHARSET':str(root/'models'/assets['charset']['name']),'NDLOCR_INTRA_THREADS':'1'}
            assert all(executed[k]==v for k,v in command.items()) and executed['exit_code']==0 and executed['frozen_files_after_unchanged']
            loaded=json.loads(data[f'run/part-results-{model}-v1/model-loaded.json']);assert loaded['loaded_session'] and loaded['model']==nn['model']
            rows=json.loads(data[f'run/part-results-{model}-v1/results.json']);assert [r['id'] for r in rows]==[r['id'] for r in selected]
            with zipfile.ZipFile(io.BytesIO(data[f'run/part-results-{model}-v1/actual-rgb-inputs.zip'])) as actual:
                for row,part in zip(rows,selected,strict=True):
                    assert 'error' not in row and actual.read(row['id']+'.rgb')==archive.read(part['member']) and row['expected_rgb_sha256']==part['rgb_sha256'] and row['orientation']=='Horizontal' and row['api_calls']==row['top_k']==1
                    verify_decode_steps(row);values[row['id']]=row;stops[row['stop_reason']]+=1
    records=[];predictions={}
    for parent in parents.values():
        model=parent['model'];ident=parent['origins'][0].split('_eval_',1)[1];routes=[[values[r['id']] for r in parts if r['parent_id']==parent['id'] and r['route']==i] for i in [0,1]];joined=[''.join(r['text'] for r in rows) for rows in routes]
        records.append({'id':parent['id'],'physical_id':ident,'model':model,'origins':parent['origins'],'raw':parent['raw'],'raw_mean_bits':parent['mean_bits'],'joined':joined,'partition_agreement':joined[0]==joined[1],'raw_prefix':[j.startswith(parent['raw']) for j in joined],'raw_subsequence':[is_subsequence(parent['raw'],j) for j in joined],'minimum_part_mean':[min(struct.unpack('!f',struct.pack('!I',r['mean_bits']))[0] for r in rows) for rows in routes]})
        for key,text in [('raw'+model,parent['raw']),('parts0_'+model,joined[0]),('parts1_'+model,joined[1])]:predictions.setdefault(key,{})[ident]=text
    assert records==json.loads(data['run/partition-diagnosis-v1.json'])
    _,controls=_packet(folder.parent/'real-vertical-additional-controls','index.json');master=json.loads(controls['tmp/real-vertical-additional-controls/additional-controls-manifest-v1.json']);scores=master['scoring_inputs'];verified=0
    for score in scores:
        assert data['run/metrics-v1/truth/'+score['id']+'.txt']==controls[score['truth']]
        for system,body in predictions.items():assert data[f'run/metrics-v1/{system}/{score["id"]}.txt'].decode()==body[score['physical_id']]
    for mode in ['none','width','strict']:
        rows=list(csv.DictReader(io.StringIO(data[f'run/metrics-v1/{mode}.csv'].decode())))
        assert len(rows)==99 and {(r['system'],r['file']) for r in rows}=={(system,s['id']+'.txt') for system in predictions for s in scores}
        for row in rows:
            expected=metric_values(data['run/metrics-v1/truth/'+row['file']].decode(),data[f'run/metrics-v1/{row["system"]}/{row["file"]}'].decode(),mode)
            assert tuple(int(row[k]) for k in ['char_errors','char_total','word_errors','word_total'])==expected;verified+=1
    summary=json.loads(data['run/partition-summary-v1.json']);agree=sum(r['model']=='30' and r['partition_agreement'] for r in records);extensions=sum(r['model']=='30' and r['partition_agreement'] and all(r['raw_prefix']) and len(r['joined'][0])>len(r['raw']) for r in records)
    assert summary['complete'] and summary['raw_API_calls']==len(values)==pre['raw_API_calls_planned']==117 and summary['API_errors']==0 and summary['all_frozen_files_after_unchanged']
    assert summary['independent_CSV_rows']==verified==297 and summary['stop_counts']==dict(stops) and summary['model30_agreeing_partitions']==agree and summary['model30_agreement_with_full_raw_prefix_extension']==extensions
    assert summary['new_accuracy_gain'] is False and summary['production_policy_changed'] is False and summary['audit_script_sha256']==sha(data['run/audit_parts_v1.py'])
    return {'raw_API_calls':len(values),'primary_model_RGB_inputs':len(parents),'original_Page_routes':sum(len(r['origins']) for r in parents.values()),'independent_crop_pixels':pixels,'verified_CSV_rows':verified,'model30_agreeing_partitions':agree,'model30_agreement_with_full_raw_prefix_extension':extensions,'new_model_calls_in_replay':0,'new_accuracy_gain':False}


if __name__=='__main__':
    print(json.dumps(verify(Path(__file__).resolve().parents[1]/'docs/benchmarks/real-vertical-primary-partition-probe'),indent=2))
