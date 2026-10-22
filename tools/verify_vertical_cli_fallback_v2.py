"""Authenticate FVC v2 sources and controls without model inference."""
import csv
import gzip
import io
import json
from pathlib import Path
import re
import struct
from PIL import Image, ImageChops
from verify_additional_real_baseline import _packet
from vertical_page_matrix import sha


def verify_before(folder):
    folder=Path(folder);index,data=_packet(folder,'before-nn-v1.index.json')
    before=json.loads(data['evidence/before-NN-v1.json']);generation=json.loads(data['evidence/generation-before-render.json']);complete=json.loads(data['evidence/generation-complete.json']);rows=json.loads(data['evidence/new-manifest.json'])['inputs']
    original_root=str(Path(before['frozen_candidate_cli']['path']).parents[2])+'/'
    def control_blob(path):
        assert path.startswith(original_root+'tmp/vertical-cli-fallback-v2/inputs/')
        return data[path[len(original_root):]]
    assert before['NN_calls']==before['model_loads']==index['NN_calls_at_freeze']==0
    assert len(rows)==72 and len({r['id'] for r in rows})==72
    assert sha(data['protocol/vertical_cli_fallback_v2_controls.md']) in before['files_sha256'].values()
    assert sha(data['evidence/controls-spec-before-render.md'])==generation['spec_sha256']
    assert sha(data['evidence/generate_v1.py'])==generation['generator_sha256']
    assert complete['NN_calls']==complete['model_loads']==0 and complete['independently_reconstructed_vertical_glyphs']==4392
    known=json.loads(gzip.decompress(data['evidence/known1239-headers-before-NN.json.gz']))
    known_path=folder.parent/'vertical-cli-fallback/full-matrix-v1/bundle.json';assert sha(known_path.read_bytes())==index['known1239_bundle_sha256']
    original=json.loads(known_path.read_text())['physical_inputs']
    keys=['id','width','height','image_sha256','rgb_sha256'];assert known==[{k:r[k] for k in keys} for r in original] and len(known)==1239
    candidate=before['frozen_candidate_cli'];test=before['frozen_candidate_test'];baseline=before['reused_fresh_FVB_baseline']
    assert candidate['sha256']!=baseline['cli']['sha256']
    assert sha(data['evidence/onnx-cli-build-retry.jsonl'])==candidate['build_stdout_sha256']
    assert sha(data['evidence/onnx-lib-build-retry.jsonl'])==test['build_stdout_sha256']
    def build_events(name):return [json.loads(line) for line in data[name].decode().splitlines() if line.startswith('{')]
    for receipt,name in [(candidate,'evidence/onnx-cli-build-retry.jsonl'),(test,'evidence/onnx-lib-build-retry.jsonl'),(baseline['cli'],'baseline-evidence/cli-regression-baseline-build.jsonl')]:
        event=receipt['compiler_artifact'];assert event['fresh'] is False and 'onnx' in event['features'] and event in build_events(name)
    for prefix,files in [('candidate-source/',before['source_files']),('baseline-source/',before['baseline_source_files'])]:
        for name,digest in files.items():assert sha(data[prefix+name])==digest,name
    assert candidate['source_files']==before['source_files'] and baseline['source_files']==before['baseline_source_files']
    source=data['candidate-source/src/app.rs'].decode();old=data['baseline-source/src/app.rs'].decode();normalize=lambda text:re.sub(r'\s','',text)
    old_loop=old[old.index('        let bounds: Vec<_>',old.index('    fn recognize_for_page_batch(')):old.index('\n    fn recognize_initial_batch(')]
    new_loop=source[source.index('        let bounds: Vec<_>',source.index('    fn finish_for_page_batch(')):source.index('\n    fn recognize_initial_batch(')]
    assert normalize(old_loop)==normalize(new_loop)
    initial=lambda text:text[text.index('    fn recognize_initial_batch('):text.index('    fn recognize_batch_on_pool_or_100(')]
    assert initial(old)==initial(source)
    assert 'cached_parseq.finish_for_page_batch(crops, results, &args)' in source
    cohorts={name:sum(r['cohort']==name for r in rows) for name in {r['cohort'] for r in rows}}
    assert cohorts=={'new_fvc2_vertical_text':36,'new_fvc2_repeated':12,'new_fvc2_portrait_horizontal':12,'new_fvc2_nonletter':12}
    glyphs=0;hashes=[]
    for row in rows:
        for key in ['image','rgb','truth']:assert sha(control_blob(row[key]))==row[key+'_sha256']
        with Image.open(io.BytesIO(control_blob(row['image']))) as im:image=im.convert('RGB')
        assert image.size==(row['width'],row['height']) and image.tobytes()==control_blob(row['rgb'])
        hashes.extend(row[k] for k in ['image_sha256','rgb_sha256'])
        truth=control_blob(row['truth']).decode()
        if row['cohort'] in ['new_fvc2_vertical_text','new_fvc2_repeated']:
            assert len(row['glyph_boxes'])==len(truth)
            leftover=image.copy()
            for box in row['glyph_boxes']:
                x0,y0,x1,y1=box;assert 0<=x0<x1<=image.width and 0<=y0<y1<=image.height
                crop=image.crop(box);ink=ImageChops.difference(crop,Image.new('RGB',crop.size,'white'))
                assert ink.getbbox()==(0,0,crop.width,crop.height)
                leftover.paste('white',box);glyphs+=1
            assert ImageChops.difference(leftover,Image.new('RGB',image.size,'white')).getbbox() is None
        if row['cohort']=='new_fvc2_vertical_text':assert len(truth) in generation['lengths'] and truth==generation['paragraph'][:len(truth)]
        if row['cohort']=='new_fvc2_repeated':assert len(truth)==69
        if row['cohort']=='new_fvc2_portrait_horizontal':assert image.height>=8*image.width and truth=='\n'.join([row['row_text']]*row['rows'])
        if row['cohort']=='new_fvc2_nonletter':assert truth==''
    assert glyphs==4392 and len(set(hashes))==144
    assert not set(hashes)&{r[k] for r in known for k in ['image_sha256','rgb_sha256']}
    return {'complete':True,'archive_sha256':index['sha256'],'new_inputs':72,'known_inputs':1239,'annotated_glyphs':glyphs,'hash_overlap':0,'NN_calls_at_freeze':0,'new_model_calls_in_replay':0,'existing_finish_loop_and_initial_batch_unchanged':True,'adopted':False}


def verify_known(folder):
    folder=Path(folder);before_result=verify_before(folder)
    index,data=_packet(folder,'known-finish-v1.index.json');_,frozen=_packet(folder,'before-nn-v1.index.json')
    assert index['before_NN_packet_sha256']==before_result['archive_sha256']
    before=json.loads(frozen['evidence/before-NN-v1.json']);bits=lambda value:struct.pack('!f',value)
    inputs={1:'fvc_arial_43_pad4.png',2:'fvc_arial_43_pad4.png',3:'known-uncertain-arial30.png',4:'known-fva-arial12.png'}
    all_settings=0;pixels_checked=0;failed=0;changed=False
    for attempt in [1,2,3,4]:
        prefix=f'run/known-finish-v{attempt}/';command=json.loads(data[prefix+'command.json']);pre=json.loads(data[prefix+'before.json'])
        assert command['files_sha256']==pre['files_sha256'] and command['frozen_bytes_after_unchanged']
        assert all(command['files_sha256'].get(n)==h for n,h in before['files_sha256'].items())
        assert command['argv'][0]==before['frozen_candidate_test']['path']
        assert command['argv'][1:]==['--ignored','--exact','app::vertical_cli_finish_known::accepted_vertical_finishes_like_existing_page_batch_with_quality_options','--nocapture','--test-threads=1']
        assert command['env']['NDLOCR_INTRA_THREADS']==command['env']['NDLOCR_PARSEQ_PARALLELISM']=='1'
        script=f'run_known_finish_v{attempt}.py';assert sha(data['evidence/'+script]) in command['files_sha256'].values()
        raw=data['known-inputs/'+inputs[attempt]];assert sha(raw) in command['files_sha256'].values()
        if attempt==1:
            assert command['exit_code']==101 and not command['complete'] and b'charset not found' in data[prefix+'run.stderr'];failed+=1;continue
        records=json.loads(data[prefix+'results/results.json']);assert len(records)==3
        assert [(r['quality_boost'],bits(r['quality_boost_min_delta'])) for r in records]==[(q,bits(d)) for q,d in [(False,.03),(True,.03),(True,-100.0)]]
        with Image.open(io.BytesIO(raw)) as im:image=im.convert('RGB')
        for ordinal,row in enumerate(records,1):
            assert row['actual_text']==row['expected_text'] and row['actual_mean_bits']==row['expected_mean_bits']
            crop=image.crop(row['bbox']);assert crop.size==(row['crop_width'],row['crop_height'])
            assert crop.tobytes()==data[prefix+f'results/crop-{ordinal}.rgb'];pixels_checked+=1
        effect=any(r['quality_boost'] and (r['actual_text']!=r['initial_text'] or r['actual_mean_bits']!=r['initial_mean_bits']) for r in records)
        all_settings+=3
        if attempt<4:
            assert not effect and command['exit_code']==101 and not command['complete'] and b'known quality option must change' in data[prefix+'run.stderr'];failed+=1
        else:
            assert effect and command['exit_code']==0 and command['complete'] and b'1 passed; 0 failed;' in data[prefix+'run.stdout'];changed=True
    for name in ['known-uncertain-arial30','known-fva-arial12']:
        origin=json.loads(data['evidence/'+name+'-origin.json']);assert sha(data['known-inputs/'+name+'.png'])==origin['image_sha256']==origin['original']['image_sha256']
    literal=json.loads(data['run/known-cli-v1/command.json']);assert literal['complete'] and literal['exit_code']==0 and literal['frozen_files_after_unchanged']
    assert literal['env']['NDLOCR_TEST_CLI_BINARY']==before['frozen_candidate_cli']['path'] and literal['CLI_processes_planned']==2
    assert b'1 passed; 0 failed;' in data['run/known-cli-v1/run.stdout']
    assert sha(data['known-cli-test.rs'])==before['source_files']['tests/vertical_fallback_cli_tests.rs']
    return {'complete':True,'archive_sha256':index['sha256'],'successful_settings':3,'all_recorded_settings':all_settings,'independent_crop_RGBs':pixels_checked,'preserved_failed_attempts':failed,'quality_changes_initial':changed,'known_literal_CLI_processes':2,'new_accuracy_gain':False,'new_model_calls_in_replay':0}


def verify_option_plan(folder):
    from vertical_cli_fallback_v2_options import option_plan,option_command
    folder=Path(folder);result=verify_before(folder)
    revision=2 if (folder/'options-plan-before-nn-v2.index.json').is_file() else 1
    index,data=_packet(folder,f'options-plan-before-nn-v{revision}.index.json');_,frozen=_packet(folder,'before-nn-v1.index.json')
    meta=json.loads(data[f'evidence/option-plan-before-NN-v{revision}.meta.json']);plans=json.loads(data[f'evidence/option-plan-before-NN-v{revision}.json'])
    assert sha(data[f'evidence/option-plan-before-NN-v{revision}.json'])==meta['option_plan_sha256']
    assert meta['source_before_NN_packet_sha256']==result['archive_sha256']
    assert meta['option_matrix_NN_calls']==index['option_matrix_NN_calls']==0
    for name,digest in meta['source_files_sha256'].items():assert sha(data[name])==digest
    repo=Path(__file__).resolve().parents[1]
    assert sha((repo/'tools/vertical_cli_fallback_v2_options.py').read_bytes())==meta['source_files_sha256']['tools/vertical_cli_fallback_v2_options.py']
    before=json.loads(frozen['evidence/before-NN-v1.json']);manifest=frozen['evidence/new-manifest.json']
    assert sha(manifest)==meta['input72_manifest_sha256']
    expected=option_plan(json.loads(manifest)['inputs']);original_root=Path(before['frozen_candidate_cli']['path']).parents[2]
    out=original_root/f'tmp/vertical-cli-fallback-v2/options-matrix-v{revision}'
    assets=json.loads((repo/'docs/benchmarks/vertical-page-fallback/bundle.json').read_text())['assets']
    for row in expected:
        label='-'.join([row['variant'],row['profile'],row['id'],'deim'+str(row['deim']).lower(),'cascade'+str(row['cascade']).lower()]);body=out/'predictions'/(label+'.txt')
        binary=before['reused_fresh_FVB_baseline']['cli']['path'] if row['variant']=='baseline' else before['frozen_candidate_cli']['path']
        row.update({'label':label,'body':str(body),'argv':option_command(row,binary,original_root/'models',assets,body)})
    assert plans==expected and len(plans)==384 and len({r['label'] for r in plans})==384
    if revision==2:assert json.loads(data['tests/fixtures/cli_options/v2-plan-argv.json'])==[r['argv'] for r in plans]
    return {'complete':True,'archive_sha256':index['sha256'],'planned_commands':384,'planned_CSV_rows':1152,'option_matrix_NN_calls':0,'gate_required_before_execution':True}


def verify_disabled(folder):
    folder=Path(folder);before_result=verify_before(folder)
    index,data=_packet(folder,'feature-disabled-v1.index.json');_,frozen=_packet(folder,'before-nn-v1.index.json')
    assert index['source_before_NN_packet_sha256']==before_result['archive_sha256']
    before=json.loads(frozen['evidence/before-NN-v1.json']);pre=json.loads(data['run/before.json']);summary=json.loads(data['run/summary.json']);commands=json.loads(data['run/commands.json'])
    assert all(pre['files_sha256'].get(n)==h for n,h in before['files_sha256'].items())
    assert sha(data['evidence/check_feature_disabled_cli_v1.py']) in pre['files_sha256'].values()
    assert summary['new_NN_calls']==summary['model_loads']==0 and summary['frozen_bytes_after_unchanged']
    binaries={variant:json.loads(data['evidence/default-cli-'+variant+'-frozen.json']) for variant in ['baseline','candidate']}
    for variant,meta in binaries.items():
        name='evidence/default-cli-build.jsonl' if variant=='candidate' else 'baseline-evidence/default-cli-baseline-build.jsonl'
        events=[json.loads(line) for line in data[name].decode().splitlines() if line.startswith('{')]
        event=meta['compiler_artifact'];assert event in events and event['fresh'] is False and 'onnx' not in event['features']
        assert pre['files_sha256'][meta['path']]==meta['sha256']
        if variant=='candidate':assert sha(data[name])==meta['build_stdout_sha256']
    assert len(commands)==16 and len({(r['variant'],r['asset'],r['case'],r['deim']) for r in commands})==16
    values={}
    for row in commands:
        variant,asset,case,deim=(row[k] for k in ['variant','asset','case','deim'])
        assert variant in binaries and asset in ['missing','dummy'] and case in ['body','white'] and type(deim) is bool
        label=f'{variant}-{asset}-{case}-{deim}';stdout=data['run/'+label+'.stdout'];stderr=data['run/'+label+'.stderr'];body=data.get('run/'+label+'.txt')
        assert sha(stdout)==row['stdout_sha256'] and sha(stderr)==row['stderr_sha256']
        assert (body is not None)==row['body_exists'] and (sha(body) if body is not None else None)==row['body_sha256']
        assert row['binary_sha256']==binaries[variant]['sha256'] and row['argv'][0]==binaries[variant]['path']
        assert row['argv'][1]=='recognize-page' and row['argv'][row['argv'].index('--use-deim-detection')+1]==str(deim).lower()
        image=data['inputs/arial121.png'] if case=='body' else data['run/white.png'];assert sha(image)==row['image_sha256']
        if case=='body':
            assert row['exit_code']==1 and stdout==b'' and body is None
            assert (b'Error: file not found' if asset=='missing' else b'Error: onnx feature is disabled') in stderr
        else:assert row['exit_code']==0 and stdout==b'\n' and body==b''
        values[variant,asset,case,deim]=(row['exit_code'],stdout,stderr,body)
    for asset in ['missing','dummy']:
        for case in ['body','white']:
            for deim in [False,True]:assert values['baseline',asset,case,deim]==values['candidate',asset,case,deim]
    return {'complete':True,'archive_sha256':index['sha256'],'CLI_processes':16,'compared_cases':8,'identical_exit_stdout_stderr_body':True,'NN_calls':0,'new_model_calls_in_replay':0}


def verify_new72(folder):
    from collections import Counter
    from vertical_page_matrix import cli_command
    from verify_low_stop_evidence import metric_values
    folder=Path(folder);before_result=verify_before(folder);known_result=verify_known(folder)
    index,data=_packet(folder,'new72-cli-matrix-v1.index.json');_,frozen=_packet(folder,'before-nn-v1.index.json');_,known=_packet(folder,'known-finish-v1.index.json')
    assert index['source_before_NN_packet_sha256']==before_result['archive_sha256'] and index['known_packet_sha256']==known_result['archive_sha256']
    before=json.loads(frozen['evidence/before-NN-v1.json']);inputs=json.loads(frozen['evidence/new-manifest.json'])['inputs'];job=json.loads(data['complete/job.json']);summary=json.loads(data['complete/summary.json'])
    assert job['complete'] and job['frozen_materials_after_unchanged'] and job['image_results']==576 and job['verified_CSV_rows']==1728
    assert sha(data['evidence/run_new72_matrix_v2.py'])==job['driver_sha256'] and sha(frozen['evidence/new-manifest.json'])==job['manifest_sha256']
    assert data['evidence/run_new72_matrix_v1.py'].replace(b"out=root/'new72-matrix-v1'",b"out=root/'new72-matrix-v2'")==data['evidence/run_new72_matrix_v2.py']
    assert sha(known['run/known-finish-v4/command.json'])==job['known_regression_receipt_sha256']
    assert job['plans']['baseline']['frozen']==before['reused_fresh_FVB_baseline']
    assert job['plans']['candidate']['frozen']=={'cli':before['frozen_candidate_cli'],'test':before['frozen_candidate_test'],'source_files':before['source_files']}
    original_root=Path(before['frozen_candidate_cli']['path']).parents[2];repo=Path(__file__).resolve().parents[1]
    for name in ['tools/vertical_page_matrix.py','tools/verify_low_stop_evidence.py','tools/eval_ocr_metrics.py']:
        assert sha((repo/name).read_bytes())==before['files_sha256'][str(original_root/name)]
    assets=json.loads((repo/'docs/benchmarks/vertical-page-fallback/bundle.json').read_text())['assets'];assert job['assets']==assets
    assert job['runtime_environment']['NDLOCR_INTRA_THREADS']==job['runtime_environment']['NDLOCR_PARSEQ_PARALLELISM']=='1'
    assert job['internal_forward_calls'] is None and job['model_session_loads'] is None
    receipts=[json.loads(raw) for name,raw in data.items() if name.startswith('complete/commands/') and name.endswith('.json')]
    assert len(receipts)==len(job['commands'])==579 and all(r['exit_code']==0 for r in receipts)
    assert Counter(tuple(r['argv']) for r in receipts)==Counter(tuple(r['argv']) for r in job['commands'])
    first=job['commands'][0]['argv'];out=Path(first[first.index('--output-txt')+1]).parents[2]
    assert out.parent==original_root/'tmp/vertical-cli-fallback-v2' and out.name=='new72-matrix-v2'
    predictions={};wanted=[]
    for variant in ['baseline','candidate']:
        binary=job['plans'][variant]['frozen']['cli']['path']
        for mode in ['deim','bands']:
            for cascade in [True,False]:
                route=mode+'_cascade_'+str(cascade).lower();key=variant+'_'+route;predictions[key]={}
                for row in inputs:
                    ident=row['id'];body=data[f'complete/predictions/{key}/{ident}.txt'];label=key+'_'+ident
                    assert data['complete/commands/'+label+'.stdout']==body+b'\n'
                    predictions[key][ident]=body.decode()
                    wanted.append(cli_command(binary,row['image'],original_root/'models',assets,mode,out/'predictions'/key/(ident+'.txt'))+['--enable-cascade',str(cascade).lower()])
    actual_cli=[r['argv'] for r in receipts if r['argv'][0] in {before['frozen_candidate_cli']['path'],before['reused_fresh_FVB_baseline']['cli']['path']}]
    assert len(actual_cli)==576 and Counter(map(tuple,actual_cli))==Counter(map(tuple,wanted))
    verified=0;comparisons=[];gains=[];regressions=[]
    for normalization in ['none','width','strict']:
        rows=list(csv.DictReader(io.StringIO(data['complete/metrics/'+normalization+'.csv'].decode())))
        assert len(rows)==576 and {(r['system'],r['file']) for r in rows}=={(system,r['id']+'.txt') for system in predictions for r in inputs}
        for row in inputs:
            truth=data['complete/metrics/truth/'+row['id']+'.txt'];key=row['truth'][len(str(original_root))+1:]
            assert truth==frozen[key] and sha(truth)==row['truth_sha256']
        for row in rows:
            ident=row['file'][:-4];truth=data['complete/metrics/truth/'+row['file']].decode();prediction=predictions[row['system']][ident]
            expected=metric_values(truth,prediction,normalization)
            assert tuple(int(row[k]) for k in ['char_errors','char_total','word_errors','word_total'])==expected;verified+=1
        indexed={(r['system'],r['file']):r for r in rows}
        for row in inputs:
            for mode in ['deim','bands']:
                for cascade in [True,False]:
                    route=mode+'_cascade_'+str(cascade).lower();ident=row['id'];a=indexed['baseline_'+route,ident+'.txt'];b=indexed['candidate_'+route,ident+'.txt']
                    entry={'normalization':normalization,'id':ident,'cohort':row['cohort'],'route':route,'before':a,'after':b,'body_unchanged':predictions['baseline_'+route][ident]==predictions['candidate_'+route][ident]};comparisons.append(entry)
                    if any(int(b[k])>int(a[k]) for k in ['char_errors','word_errors']):regressions.append(entry)
                    if normalization=='none' and row['cohort']=='new_fvc2_vertical_text' and int(b['char_errors'])<int(a['char_errors']) and int(b['word_errors'])<=int(a['word_errors']):gains.append(entry)
    assert comparisons==json.loads(data['complete/comparisons.json']) and gains==json.loads(data['complete/new-body-gains.json']) and regressions==json.loads(data['complete/regressions.json'])
    assert verified==1728 and summary['new_body_raw_gain_pairs']==job['new_body_raw_gain_pairs']==len(gains)
    assert summary['regression_pairs']==job['regression_pairs']==len(regressions)
    passed=not regressions and bool(gains);assert passed==job['matrix_gate_pass']==summary['matrix_gate_pass']
    assert all(r['body_unchanged'] for r in comparisons if r['cohort']!='new_fvc2_vertical_text')
    stopped=json.loads(data['stopped/job.json']);stop_meta=json.loads(data['evidence/new72-stopped-attempt-v1.json'])
    assert not stopped['complete'] and len(stopped['commands'])==129 and all(r['exit_code']==0 for r in stopped['commands'])
    assert stop_meta['process_exit_code']==120 and stop_meta['accepted_complete_results']==0 and data['evidence/new72-matrix-v1.stderr']==b''
    return {'complete':True,'archive_sha256':index['sha256'],'CLI_processes':576,'verified_CSV_rows':verified,'raw_gain_pairs':len(gains),'unique_body_images_improved':len({r['id'] for r in gains}),'regressions':len(regressions),'accepted_stopped_results':0,'matrix_gate_pass':passed,'new_model_calls_in_replay':0}


def verify_parser_preflight(folder):
    folder=Path(folder);verify_option_plan(folder)
    index,data=_packet(folder,'option-parser-preflight-v2.index.json');plan_index,plan_data=_packet(folder,'options-plan-before-nn-v2.index.json')
    assert index['plan_v2_sha256']==plan_index['sha256']
    plans=json.loads(plan_data['evidence/option-plan-before-NN-v2.json']);fixture=json.loads(data['tests/fixtures/cli_options/v2-plan-argv.json'])
    assert fixture==[r['argv'] for r in plans] and len(fixture)==384
    assert b'10 passed; 0 failed;' in data['evidence/options-real-parser-green.stdout']
    unique={}
    for row in plans:unique.setdefault((row['variant'],row['profile'],row['deim'],row['cascade']),row)
    success=json.loads(data['run/option-parse-preflight-v3/job.json']);before=json.loads(data['run/option-parse-preflight-v3/before.json'])
    assert success['complete'] and success['CLI_processes']==len(unique)==48 and success['frozen_files_after_unchanged']
    assert success['files_sha256']==before['files_sha256'] and success['new_NN_calls']==success['model_loads']==0
    assert sha(data['evidence/run_option_parse_preflight_v3.py']) in success['files_sha256'].values()
    for index_,(row,command) in enumerate(zip(unique.values(),success['commands'],strict=True)):
        argv=list(row['argv']);missing=command['argv'][command['argv'].index('--image')+1];assert Path(missing).name=='permanently-absent-input.png';argv[argv.index('--image')+1]=missing
        assert command['argv']==argv and command['exit_code']==1
        stdout=data[f'run/option-parse-preflight-v3/{index_}.stdout'];stderr=data[f'run/option-parse-preflight-v3/{index_}.stderr']
        assert stdout==b'' and b'No such file or directory (os error 2)' in stderr
        assert sha(stdout)==command['stdout_sha256'] and sha(stderr)==command['stderr_sha256']
    failed=json.loads(data['run/option-parse-preflight-v2/job.json']);assert not failed['complete'] and len(failed['commands'])==1 and failed['commands'][0]['exit_code']==1
    assert b'AssertionError' in data['run/option-parse-preflight-v2.stderr']
    return {'complete':True,'archive_sha256':index['sha256'],'parsed_argv':384,'successful_native_vectors':48,'preserved_failed_wrapper':1,'NN_calls':0,'model_loads':0,'new_model_calls_in_replay':0}


def verify_root_before(folder):
    from verify_additional_real_partition_probe import verify as verify_partitions
    folder=Path(folder);verify_partitions(folder.parent/'real-vertical-primary-partition-probe')
    ri,data=_packet(folder/'root-bridge-v1','baseline-before-green-v1.index.json');di,default=_packet(folder/'root-bridge-v1','default-baseline-v1.index.json')
    assert di['baseline_source_packet_sha256']==ri['sha256']
    vi,v2=_packet(folder,'before-nn-v1.index.json');gi,geometry=_packet(folder.parent/'real-vertical-primary-partition-probe','before-nn-v1.index.json');si,stop=_packet(folder.parent/'real-vertical-additional-stop-probe','before-nn-v1.index.json')
    assert ri['FVCv2_before_packet_sha256']==vi['sha256'] and ri['geometry_source_before_packet_sha256']==gi['sha256']
    gbridge=json.loads(geometry['evidence/source-bridge-v1.json']);gsource={}
    assert gbridge['packet_SHA256']=={'FVCv2':vi['sha256'],'stop':si['sha256']}
    for name,meta in gbridge['source_files'].items():
        raw={'FVCv2':v2,'stop':stop,'this':geometry}[meta['packet']][meta['member']];assert sha(raw)==meta['sha256'];gsource[name]=raw
    bridge=json.loads(data['evidence/source-bridge-v1.json']);frozen=json.loads(data['evidence/root-baseline-frozen-v1.json']);source={}
    assert bridge['geometry_before_packet_sha256']==gi['sha256'] and set(bridge['source_files'])==set(frozen['source_files'])
    for name,meta in bridge['source_files'].items():
        raw={'geometry':gsource,'this':data}[meta['packet']][meta['member']];assert sha(raw)==meta['sha256']==frozen['source_files'][name];source[name]=raw
    assert 'vertical_fallback::try_prepared_if_empty(' not in source['src/app.rs'].decode()
    events=[json.loads(line) for line in data['evidence/root-baseline-build-v1.jsonl'].decode().splitlines() if line.startswith('{')]
    assert sha(data['evidence/root-baseline-build-v1.jsonl'])==frozen['build_stdout_sha256']
    assert len(frozen['own_artifacts'])==3
    for event in frozen['own_artifacts']:assert event in events and event['fresh'] is False and 'onnx' in event['features']
    for key,meta in frozen['binaries'].items():assert meta['compiler_artifact'] in frozen['own_artifacts'] and meta['compiler_artifact']['executable'] is not None
    root=Path(frozen['binaries']['bin']['path']).parents[2];pre=json.loads(data['baseline-red/before-NN.json']);run=json.loads(data['baseline-red/command.json'])
    assert not pre['complete'] and run['complete'] and run['files_sha256']==pre['files_sha256'] and run['frozen_files_after_unchanged']
    assert run['exit_code']==101 and run['expected_literal_failures']==run['successful_CLI_processes']==2 and b'2 failed;' in data['baseline-red/run.stdout']
    vb=json.loads(v2['evidence/before-NN-v1.json'])
    for name,digest in pre['files_sha256'].items():
        rel=str(Path(name).relative_to(root))
        if rel in source:raw=source[rel]
        elif rel.startswith('models/'):assert vb['files_sha256'][name]==digest;continue
        elif name in {m['path'] for m in frozen['binaries'].values()}:assert next(m['sha256'] for m in frozen['binaries'].values() if m['path']==name)==digest;continue
        elif rel=='docs/vertical_cli_fallback_v2_root_bridge.md':raw=data['protocol/vertical_cli_fallback_v2_root_bridge.md']
        else:raw=data['evidence/'+Path(rel).name]
        assert sha(raw)==digest
    expected_argv=[frozen['binaries']['test']['path'],'--ignored','--nocapture','--test-threads=1'];assert pre['argv']==run['argv']==expected_argv
    candidate=json.loads(data['known-candidate-precheck/command.json'])
    assert candidate['argv']==expected_argv and candidate['exit_code']==0 and candidate['CLI_processes']==2 and candidate['known_only'] and candidate['Root_runtime_integrated'] is False
    assert candidate['candidate_sha256']==vb['frozen_candidate_cli']['sha256'] and candidate['test_sha256']==frozen['binaries']['test']['sha256']
    assert b'2 passed; 0 failed;' in data['known-candidate-precheck/run.stdout']
    case=json.loads(source['tests/fixtures/vertical_fallback_page/case.json']);expected_body=case['expected'].encode()
    ground_truth=source['tests/fixtures/vertical_fallback_page/'+case['truth']]
    assert len(case['expected'])==120 and len(ground_truth.decode())==121
    assert sha(expected_body)==case['expected_sha256'] and sha(ground_truth)==case['truth_sha256']
    assert ground_truth.decode().count('\u3000')==1 and ground_truth.decode().replace('\u3000','')==case['expected']
    for label,binary,environment in [('baseline-red',frozen['binaries']['bin']['path'],pre['env']),('known-candidate-precheck',vb['frozen_candidate_cli']['path'],candidate['env'])]:
        assert environment['NDLOCR_TEST_CLI_BINARY']==binary and environment['NDLOCR_INTRA_THREADS']==environment['NDLOCR_PARSEQ_PARALLELISM']=='1'
        for deim in [False,True]:
            prefix=f'{label}/evidence/deim-{str(deim).lower()}/';command=json.loads(data[prefix+'command.json']);argv=command['argv'];body=data[prefix+'body.txt']
            assert command['binary']==binary and command['exit_code']==0 and command['deim']==deim
            assert argv[:2]==['recognize-page','--image'] and argv[2]==str(root/'tests/fixtures/vertical_fallback_page'/case['image'])
            assert argv[argv.index('--use-deim-detection')+1]==str(deim).lower() and argv[argv.index('--enable-cascade')+1]=='false' and argv[argv.index('--structure-rules')+1]=='false'
            assert data[prefix+'stdout']==body+b'\n'
            assert (body==expected_body)==(label=='known-candidate-precheck')
    default_meta=json.loads(default['evidence/root-default-baseline-frozen-v1.json']);assert default_meta['source_files']==frozen['source_files'] and default_meta['NN_calls']==default_meta['model_loads']==0
    raw=default['evidence/root-default-baseline-build-v1.jsonl'];assert sha(raw)==default_meta['build_stdout_sha256'];events=[json.loads(line) for line in raw.decode().splitlines() if line.startswith('{')]
    assert len(default_meta['own_artifacts'])==2
    for event in default_meta['own_artifacts']:assert event in events and event['fresh'] is False and 'onnx' not in event['features']
    return {'baseline_literal_Red':2,'candidate_known_precheck_Green':2,'baseline_CLI_processes':2,'candidate_CLI_processes':2,'fixture_GT_scalars':121,'fixture_literal_scalars':120,'Root_runtime_integrated':False,'default_baseline_NN_calls':0,'new_model_calls_in_replay':0}


def verify_options(folder):
    from collections import Counter
    from verify_low_stop_evidence import metric_values
    folder=Path(folder);new72=verify_new72(folder);plan_result=verify_option_plan(folder);verify_parser_preflight(folder)
    index,data=_packet(folder,'options-cli-matrix-v2.index.json')
    before_index,frozen=_packet(folder,'before-nn-v1.index.json');plan_index,plan_data=_packet(folder,'options-plan-before-nn-v2.index.json');_,parser_data=_packet(folder,'option-parser-preflight-v2.index.json')
    assert index['before_packet_sha256']==before_index['sha256'] and index['new72_packet_sha256']==new72['archive_sha256'] and index['plan_v2_sha256']==plan_index['sha256']
    plans=json.loads(plan_data['evidence/option-plan-before-NN-v2.json']);job=json.loads(data['complete/job.json']);pre=json.loads(data['complete/before-NN.json']);summary=json.loads(data['complete/summary.json'])
    before=json.loads(frozen['evidence/before-NN-v1.json']);original_root=Path(before['frozen_candidate_cli']['path']).parents[2]
    assert job['complete'] and not pre['complete'] and not pre['commands'] and pre['executed_CLI_processes']==0
    assert job['files_sha256']==pre['files_sha256'] and job['frozen_files_after_unchanged']
    assert all(job['files_sha256'].get(n)==h for n,h in before['files_sha256'].items())
    assert job['verified_new72_gate']==new72 and job['verified_option_plan']==plan_result
    assert data['complete/plan.json']==plan_data['evidence/option-plan-before-NN-v2.json']
    bindings={
        'run_options_matrix_v2.py':data['evidence/run_options_matrix_v2.py'],
        'options-preflight-reader-v2.py':data['evidence/options-preflight-reader-v2.py'],
        'options-matrix-v2/plan.json':data['complete/plan.json'],
        'option-parse-preflight-v3/job.json':parser_data['run/option-parse-preflight-v3/job.json'],
        'run_option_parse_preflight_v3.py':parser_data['evidence/run_option_parse_preflight_v3.py'],
    }
    for name,raw in bindings.items():assert job['files_sha256'][str(original_root/'tmp/vertical-cli-fallback-v2'/name)]==sha(raw)
    assert job['runtime_environment']['NDLOCR_INTRA_THREADS']==job['runtime_environment']['NDLOCR_PARSEQ_PARALLELISM']=='1'
    assert job['internal_forward_calls'] is None and job['model_session_loads'] is None and job['new_accuracy_gain'] is False
    assert summary=={k:v for k,v in job.items() if k not in ['files_sha256','commands']}
    receipts=[json.loads(raw) for name,raw in data.items() if name.startswith('complete/commands/') and name.endswith('.json')]
    assert len(receipts)==len(job['commands'])==387 and all(r['exit_code']==0 for r in receipts)
    assert Counter(json.dumps(r,sort_keys=True) for r in receipts)==Counter(json.dumps(r,sort_keys=True) for r in job['commands'])
    assert [r['argv'] for r in job['commands'][:384]]==[r['argv'] for r in plans]
    predictions={};empty=0
    for row in plans:
        body=data['complete/predictions/'+row['label']+'.txt'];assert data['complete/commands/'+row['label']+'.stdout']==body+b'\n'
        if row['profile']=='reject_all':assert body==b'';empty+=1
        system='_'.join([row['variant'],row['profile'],'deim',str(row['deim']).lower(),'cascade',str(row['cascade']).lower()])
        assert data[f"complete/metrics/{system}/{row['id']}.txt"]==body
        predictions.setdefault(system,{})[row['id']]=body.decode()
    inputs={r['id']:r for r in json.loads(frozen['evidence/new-manifest.json'])['inputs']};ids=list(dict.fromkeys(r['id'] for r in plans))
    assert len(ids)==8 and len(predictions)==48 and all(set(v)==set(ids) for v in predictions.values())
    for ident in ids:
        row=inputs[ident];truth=data['complete/metrics/truth/'+ident+'.txt'];assert sha(truth)==row['truth_sha256']
        assert truth==frozen[str(Path(row['truth']).relative_to(original_root))]
    comparisons=[];regressions=[];verified=0
    for normalization in ['none','width','strict']:
        out=original_root/'tmp/vertical-cli-fallback-v2/options-matrix-v2'
        command=job['commands'][384+['none','width','strict'].index(normalization)]['argv']
        wanted=[command[0],str(original_root/'tools/eval_ocr_metrics.py'),'--truth-dir',str(out/'metrics/truth')]
        for system in predictions:wanted+=['--system',system+'='+str(out/'metrics'/system)]
        wanted+=['--normalize',normalization,'--require-all','--output-csv',str(out/'metrics'/(normalization+'.csv'))]
        assert command==wanted
        rows=list(csv.DictReader(io.StringIO(data['complete/metrics/'+normalization+'.csv'].decode())))
        assert len(rows)==384 and {(r['system'],r['file']) for r in rows}=={(s,i+'.txt') for s in predictions for i in ids}
        for row in rows:
            expected=metric_values(data['complete/metrics/truth/'+row['file']].decode(),predictions[row['system']][row['file'][:-4]],normalization)
            assert tuple(int(row[k]) for k in ['char_errors','char_total','word_errors','word_total'])==expected;verified+=1
        indexed={(r['system'],r['file']):r for r in rows}
        for row in plans:
            if row['variant']!='baseline':continue
            suffix='_'.join([row['profile'],'deim',str(row['deim']).lower(),'cascade',str(row['cascade']).lower()]);a=indexed['baseline_'+suffix,row['id']+'.txt'];b=indexed['candidate_'+suffix,row['id']+'.txt']
            entry={k:row[k] for k in ['id','profile','deim','cascade']};entry.update({'normalization':normalization,'before':a,'after':b});comparisons.append(entry)
            if any(int(b[k])>int(a[k]) for k in ['char_errors','word_errors']):regressions.append(entry)
    assert comparisons==json.loads(data['complete/comparisons.json']) and regressions==json.loads(data['complete/regressions.json'])
    assert verified==job['verified_CSV_rows']==1152 and len(comparisons)==job['comparisons']==576 and empty==64
    assert job['executed_CLI_processes']==job['CLI_processes_planned']==384 and job['official_CSV_rows_planned']==1152
    assert len(regressions)==job['regressions']==index['regressions'] and job['option_gate_pass']==(not regressions)
    return {'complete':True,'archive_sha256':index['sha256'],'CLI_processes':384,'verified_CSV_rows':verified,'comparisons':len(comparisons),'regressions':len(regressions),'reject_all_empty_processes':empty,'option_gate_pass':not regressions,'new_accuracy_gain':False,'new_model_calls_in_replay':0}


if __name__=='__main__':
    import argparse
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--folder',type=Path,default=Path(__file__).resolve().parents[1]/'docs/benchmarks/vertical-cli-fallback-v2');mode=parser.add_mutually_exclusive_group();mode.add_argument('--known',action='store_true');mode.add_argument('--options-plan',action='store_true');mode.add_argument('--disabled',action='store_true');mode.add_argument('--new72',action='store_true');mode.add_argument('--parser-preflight',action='store_true');mode.add_argument('--options',action='store_true');args=parser.parse_args();verify=verify_options if args.options else verify_parser_preflight if args.parser_preflight else verify_new72 if args.new72 else verify_disabled if args.disabled else verify_option_plan if args.options_plan else verify_known if args.known else verify_before;print(json.dumps(verify(args.folder),indent=2))
