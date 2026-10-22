"""Replay additional real-column trace pixels and scores without native inference."""
import csv
import io
import json
import math
from collections import Counter
from pathlib import Path
import struct
import zipfile
from PIL import Image
from additional_real_primary_trace import same_page_output
from verify_additional_real_baseline import _packet, verify as verify_baseline
from vertical_page_matrix import safe_member, sha
from verify_low_stop_evidence import metric_values


def rejection_reasons(base,limit,routes):
    reasons=[]
    if limit<3 or not limit-1<=len(base)<=limit:reasons.append('base_not_near_capacity')
    if base and base[-1].isspace():reasons.append('base_trailing_whitespace')
    if len(routes)!=2 or any(len(v)<2 for v in routes):reasons.append('partition_cardinality')
    for parts in routes:
        for part in parts:
            if 'error' in part:reasons.append('part_error');continue
            if not part['text'].strip():reasons.append('part_empty')
            if len(part['text'])>=limit:reasons.append('part_saturated')
            confidence=struct.unpack('!f',struct.pack('!I',part['mean_bits']))[0]
            threshold=struct.unpack('!f',struct.pack('!f',.9))[0]
            if not math.isfinite(confidence) or not threshold<=confidence<=1:reasons.append('part_confidence')
    if any('error' in p for route in routes for p in route):return sorted(set(reasons))
    texts=[''.join(p['text'] for p in route) for route in routes]
    if len(texts)!=2:return sorted(set(reasons))
    text,other=texts
    if text!=other:reasons.append('partition_disagreement')
    if not text.startswith(base):reasons.append('raw_prefix_changed')
    if len(text)<=len(base):reasons.append('not_extension')
    if len({c for c in text if c.isalnum()})<4:reasons.append('insufficient_distinct_alnum')
    if [c for c in text if c.isspace()]!=[c for c in base if c.isspace()]:reasons.append('whitespace_changed')
    return sorted(set(reasons))


def verify(folder):
    folder=Path(folder);index=json.loads((folder/'trace-v1.index.json').read_text());archive=folder/safe_member(index['archive'])
    assert archive.stat().st_size==index['bytes'] and sha(archive.read_bytes())==index['sha256']
    with zipfile.ZipFile(archive) as z:
        names=z.namelist();expected={'objects/'+r['sha256'] for r in index['members'].values()}
        assert len(names)==len(set(names)) and set(names)==expected
        data={}
        for name,meta in index['members'].items():
            safe_member(name);raw=z.read('objects/'+meta['sha256']);assert len(raw)==meta['bytes'] and sha(raw)==meta['sha256'];data[name]=raw
    controls_folder=folder.parent/'real-vertical-additional-controls'
    assert verify_baseline(controls_folder)['verified_CSV_rows']==264
    _,controls=_packet(controls_folder,'index.json');_,baseline=_packet(controls_folder,'FVB-baseline-real6-v1.index.json')
    master=json.loads(controls['tmp/real-vertical-additional-controls/additional-controls-manifest-v1.json'])
    job=json.loads(data['run/job-v1.json']);before=json.loads(data['run/before-NN-v1.json']);summary=json.loads(data['run/audit-summary-v1.json'])
    assert job['complete'] and job['frozen_files_after_unchanged'] and job['GT_not_passed'] and not job['new_inference_policy']
    assert job['files_sha256']==before['files_sha256'] and not before['complete']
    assert len(job['commands'])==2 and job['actual_Page_API_results']==36
    assert job['successful_PageRecognizer_loads']==6 and job['explicit_Page_model_session_loads']==12
    assert sha(data['run/run_v1.py']) in job['files_sha256'].values()
    assert sha(data['run/audit_v1.py'])==summary['audit_script_sha256']
    source=json.loads(data['observer/source.json']);build=json.loads(data['observer/build-receipt.json'])
    assert build['exit_code']==build['unit_exit_code']==0 and build['unit_NN_calls']==0
    assert build['primary_test_artifact']['fresh'] is False and 'onnx' in build['primary_test_artifact']['features']
    events=[json.loads(line) for line in data['observer/build.jsonl'].decode().splitlines() if line.startswith('{')]
    assert build['primary_test_artifact'] in events
    assert sha(data['observer/source-v2.zip'])==source['archive_sha256']
    with zipfile.ZipFile(io.BytesIO(data['observer/source-v2.zip'])) as z:
        for name,meta in source['files'].items():assert sha(z.read(name))==meta['sha256']
    assert build['binary_sha256']=='1a5c0881f7a532b1bd02fbcd89f0f03052329f7841f747a1b3677910f34c98c3'
    assert build['binary_sha256'] in job['files_sha256'].values()
    origin=json.loads(data['run/origin-v1.json']);assert len(origin['models'])==5
    frozen_assets=json.loads(baseline['run/job.json'])['assets']
    assert {Path(n).name:h for n,h in origin['models'].items()}=={r['name']:r['sha256'] for r in frozen_assets.values()}
    # Resolve every before-NN binding to a public byte source or a fixed model/binary receipt.
    source_hashes={n:r['sha256'] for n,r in source['files'].items()}
    for name,digest in job['files_sha256'].items():
        match=None
        if '/real-vertical-rashomon-diagnostic/source/' in name:
            assert source_hashes[name.split('/real-vertical-rashomon-diagnostic/source/',1)[1]]==digest;continue
        for marker,prefix,mapping in [('/tmp/real-vertical-additional-primary-trace/','run/',data),('/tmp/real-vertical-rashomon-diagnostic/','observer/',data),('/tmp/real-vertical-additional-controls/FVB-baseline-real6-v1/','run/',baseline),('/tmp/real-vertical-additional-controls/','tmp/real-vertical-additional-controls/',controls)]:
            if marker in name:
                match=mapping.get(prefix+name.split(marker,1)[1]);break
        if match is None:
            for marker in ['/docs/','/tools/','/tests/','/tmp/vertical-page-primary/']:
                if marker in name:match=data.get('evidence/'+marker[1:]+name.split(marker,1)[1]);break
        if match is not None:assert sha(match)==digest,name
        else:assert digest in set(origin['models'].values())|{build['binary_sha256']},name
    transports=json.loads(data['transport/zip-members.json']);counts=Counter();reasons=Counter();physical={r['id']:r for r in master['inputs']};all_ids=[];predictions={};diagnoses=json.loads(data['run/trace-diagnosis-v1.json'])
    for batch,command in enumerate(job['commands'],1):
        prefix=f'run/batch{batch}-v1/';receipt=json.loads(data[prefix+'command.json'])
        assert receipt==command and receipt['exit_code']==0 and receipt['all_frozen_bytes_unchanged']
        assert receipt['argv'][0].endswith('/real-vertical-rashomon-diagnostic/trace-binary')
        assert receipt['argv'][1:]==['--ignored','--exact','infer::page_primary_trace::capture_fixed_page_primary_inputs','--nocapture','--test-threads=1']
        assert receipt['env']['NDLOCR_INTRA_THREADS']=='1'
        assert receipt['env']['NDLOCR_PRIMARY_MANIFEST'].endswith(f'/batch{batch}-v1/NN-images-only.json')
        assert receipt['env']['NDLOCR_PRIMARY_OUTPUT'].endswith(f'/batch{batch}-v1/results')
        manifest=json.loads(data[prefix+'NN-images-only.json'])['inputs'];assert len(manifest)==3 and all(set(r)=={'id','image'} for r in manifest)
        all_ids.extend(r['id'] for r in manifest)
        batch_summary=json.loads(data[prefix+'results/summary.json']);assert batch_summary['Page_API_calls']==18 and batch_summary['model_loads']==6
        batch_counts=Counter()
        for model in ['30','50','100']:
            for option in ['eval','default']:
                route=model+'_'+option;base_rows=json.loads(baseline[f'run/page/{model}/runtime{model}_{option}/results.json']);base={r['id']:r for r in base_rows}
                for input in manifest:
                    ident=input['id'];assert input['image'].endswith('/'+physical[ident]['image'])
                    key=f'{model}-{option}-{ident}';row=json.loads(data[prefix+'results/'+key+'.json']);assert row['id']==ident and row['model']==model and row['option']==option and same_page_output(row,base[ident]);counts['root_FVB_signatures_exact']+=1
                    predictions.setdefault('baseline_'+route,{})[ident]=base[ident]['text'];predictions.setdefault('trace_'+route,{})[ident]=row['text']
                    with Image.open(io.BytesIO(controls[physical[ident]['image']])) as im:image=im.convert('RGB')
                    images={};current=None;parts=[];raw=None;after=None;transport=transports[prefix+'results/'+key+'.zip']
                    assert len(transport['original_container_sha256'])==64 and transport['original_container_bytes']>0
                    for ordinal,event in enumerate(row['events']):
                        kind=event['kind'];counts[kind]+=1;batch_counts[kind]+=1
                        if 'image' in event:
                            meta=event['image'];pixels=data[transport['members'][meta['file']]];assert len(pixels)==meta['width']*meta['height']*3;images[ordinal]=(pixels,meta['width'],meta['height'])
                        if kind=='Page_primary_crop':
                            assert event['data']['using_bands'] is False;crop=image.crop(event['data']['bounds']);assert images[ordinal]==(crop.tobytes(),*crop.size);counts['independent_primary_RGB_exact']+=1
                        if kind=='raw_before_H_O':
                            assert any(e['kind']=='Page_primary_crop' and images[ordinal]==images[i] for i,e in enumerate(row['events']) if i in images);raw=event['data']
                        if kind=='O_plan':
                            current=event['data']|{'image_index':ordinal};parts=[]
                            if not current['plan'] and not current['limit']-1<=len(current['base'])<=current['limit']:reasons['base_not_near_capacity']+=1
                        if kind=='O_part':
                            assert current['plan'];pixels,w,h=images[current['image_index']];rotated=Image.frombytes('RGB',(w,h),pixels).transpose(Image.Transpose.ROTATE_90);left,right=event['data']['range'];crop=rotated.crop((left,0,right,w));assert images[ordinal]==(crop.tobytes(),*crop.size);counts['independent_O_part_RGB_exact']+=1;parts.append(event['data'])
                        if kind=='O_combine':
                            sizes=list(map(len,current['routes']));assert len(parts)==sum(sizes);routes=[parts[:sizes[0]],parts[sizes[0]:]];why=rejection_reasons(current['base'],current['limit'],routes);assert (not why)==event['data']['adopted'];reasons.update(why)
                            saved=next(r for r in diagnoses if r['key']==key and r['kind']=='O_combination');assert saved['base']==current['base'] and saved['parts']==routes and saved['reasons']==why and saved['adopted']==event['data']['adopted']
                        if kind=='after_H_O_before_sanitize':after=event['data']
                    saved=next(r for r in diagnoses if r['key']==key and r['kind']=='raw_and_final');assert saved['raw']==raw and saved['before_sanitize']==after and saved['Page_text']==row['text'];assert saved['H_stop_reason'] is None and saved['raw_decoder_stop_reason'] is None
                    counts['H_adopted' if after['recovered_by_H'] else 'H_not_adopted']+=1;counts['all_trace_RGBs']+=len(images)
        assert dict(batch_counts)==batch_summary['event_counts']
    assert all_ids==[r['id'] for r in master['inputs']] and len(set(all_ids))==6
    assert dict(counts)==summary['counts'] and dict(reasons)==summary['O_reasons']
    assert counts['PARSeq_forward_attempt']==counts['PARSeq_forward_success']==66
    verified=0
    for mode in ['none','width','strict']:
        rows=list(csv.DictReader(io.StringIO(data['run/metrics-v1/'+mode+'.csv'].decode())))
        assert len(rows)==132 and len({(r['system'],r['file']) for r in rows})==132
        for row in rows:
            alias=next(r for r in master['scoring_inputs'] if row['file']==r['id']+'.txt');truth=data['run/metrics-v1/truth/'+row['file']];assert truth==controls[alias['truth']]
            prediction=data['run/metrics-v1/'+row['system']+'/'+row['file']].decode();assert prediction==predictions[row['system']][alias['physical_id']]
            expected=metric_values(truth.decode(),prediction,mode);assert tuple(int(row[k]) for k in ['char_errors','char_total','word_errors','word_total'])==expected;verified+=1
    return {'complete':True,'archive_sha256':index['sha256'],**{k:counts[k] for k in ['root_FVB_signatures_exact','all_trace_RGBs','independent_primary_RGB_exact','independent_O_part_RGB_exact']},'verified_CSV_rows':verified,'actual_PARSeq_forward_successes':counts['PARSeq_forward_success'],'new_inference_policy':False}


if __name__=='__main__':
    import argparse
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--folder',type=Path,default=Path(__file__).resolve().parents[1]/'docs/benchmarks/real-vertical-additional-primary-trace');args=parser.parse_args();print(json.dumps(verify(args.folder),indent=2))
