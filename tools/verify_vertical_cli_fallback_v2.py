"""Authenticate FVC v2 sources and controls without model inference."""
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


if __name__=='__main__':
    import argparse
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--folder',type=Path,default=Path(__file__).resolve().parents[1]/'docs/benchmarks/vertical-cli-fallback-v2');parser.add_argument('--known',action='store_true');args=parser.parse_args();print(json.dumps(verify_known(args.folder) if args.known else verify_before(args.folder),indent=2))
