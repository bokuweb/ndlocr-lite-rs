"""Replay complete FVC worker archives and independent scores without native inference."""
from collections import Counter
import csv
import io
import json
import math
from pathlib import Path
import struct
import zipfile
from verify_additional_real_baseline import _packet
from verify_low_stop_evidence import metric_values
import vertical_cli_fallback_full_matrix as full
import vertical_page_matrix as matrix


def members(raw):
    with zipfile.ZipFile(io.BytesIO(raw)) as z:
        names=z.namelist()
        assert len(names)==len(set(names))
        for name in names:matrix.safe_member(name)
        return {n:z.read(n) for n in names}


def verify(folder,bundle=None):
    folder=Path(folder);bundle=bundle or full.FullBundle(folder)
    index,data=_packet(folder,'completed-linux-v1.index.json');bundle_sha=matrix.sha((bundle.folder/'bundle.json').read_bytes())
    assert index['bundle_sha256']==bundle_sha
    run=json.loads(data['evidence/run.json']);assert run['conclusion']=='success' and run['headSha']==index['head']
    assert len(run['jobs'])==59 and all(j['conclusion']=='success' for j in run['jobs'])
    build_raw=data['stage/build.json'];build=json.loads(build_raw);build_sha=matrix.sha(build_raw)
    assert build['complete'] and build['source_files_after_unchanged'] and build['baseline_FVB_bridge']
    assert build['bundle_sha256']==bundle_sha and build['NN_calls']==build['model_loads']==0
    omission=json.loads(data['evidence/native-omissions.json']);assert omission['all_original_stage_bytes_verified'] and omission['native_bytes_in_public_packet'] is False
    omitted=omission['omitted_native_members'];kept={n[6:]:raw for n,raw in data.items() if n.startswith('stage/') and n!='stage/build.json'}
    assert set(kept)|set(omitted)==set(build['stage_files']) and not set(kept)&set(omitted)
    for name,meta in build['stage_files'].items():
        if name in kept:assert len(kept[name])==meta['bytes'] and matrix.sha(kept[name])==meta['sha256']
        else:assert name.startswith(('binaries/','runtime/')) and omitted[name]==meta
    for name,digest in build['harness_source_files'].items():assert matrix.sha(kept['harness/'+name])==digest
    assert matrix.sha(kept['harness/tools/vertical_cli_fallback_full_matrix.py'])==build['producer_sha256']
    for variant,receipt in build['builds'].items():
        assert receipt['source_files']==bundle.index['source_files'][variant]
        raw=kept['commands/'+variant+'-build.stdout'];assert matrix.sha(raw)==receipt['build_stdout_sha256']
        events=[json.loads(line) for line in raw.decode().splitlines() if line.strip()]
        assert matrix.check_build_artifacts(events,receipt['source_manifest'])==receipt['artifacts']
        for binary in receipt['binaries'].values():assert build['stage_files'][binary['file']]=={k:binary[k] for k in ['bytes','sha256']}
    downloaded=json.loads(data['evidence/downloaded.json']);assert downloaded['run']==index['run']
    for artifact in downloaded['artifacts']:
        name='artifacts/'+artifact['name']+'.zip'
        if artifact['name']==omission['original_build_artifact']['name']:assert artifact==omission['original_build_artifact'];continue
        assert len(data[name])==artifact['bytes'] and matrix.sha(data[name])==artifact['sha256']
    workers={name:members(raw) for name,raw in data.items() if name.startswith('artifacts/') and '-result-' in name}
    metas=[json.loads(d['evidence/job.json']) for d in workers.values()]
    image_results=full.validate_complete_jobs(metas,bundle.plan,bundle_sha,build_sha)
    predictions={kind:{} for kind in full.KINDS};page={}
    def merge(target,values):
        assert not set(target)&set(values);target.update(values)
    for worker in workers.values():
        meta=json.loads(worker['evidence/job.json']);pre=json.loads(worker['evidence/before-NN.json']);kind=meta['kind'];shard=meta['shard']
        selected=full.partition_inputs(bundle.index['physical_inputs'],shard['index'],shard['total']);manifest=meta['manifest']
        assert manifest==pre['manifest'] and meta['planned_commands']==pre['planned_commands'] and not pre['complete'] and not pre['commands']
        assert [r['id'] for r in manifest['inputs']]==[r['id'] for r in selected] and all(set(r)=={'id','image'} for r in manifest['inputs'])
        assert matrix.sha((json.dumps(manifest,ensure_ascii=False,indent=2)+'\n').encode())==meta['manifest_sha256']
        work=Path(manifest['inputs'][0]['image']).parent.parent;stage=Path(meta['stage_directory']);models=Path(meta['models_dir'])
        assert all(r['image']==str(work/full.input_relative_path(s)) for r,s in zip(manifest['inputs'],selected,strict=True))
        assert meta['assets']==bundle.index['assets'] and meta['producer_sha256']==build['producer_sha256'] and meta['profile']=='release' and meta['platform'].startswith('Linux')
        assert meta['global_physical_inputs']==len(bundle.index['physical_inputs']) and meta['global_scoring_inputs']==len(bundle.index['scoring_inputs'])
        assert meta['physical_inputs']==len(selected) and meta['scoring_inputs']==sum(r['physical_id'] in shard['physical_ids'] for r in bundle.index['scoring_inputs'])
        binaries={v:{k:str(stage/b['file']) for k,b in r['binaries'].items()} for v,r in build['builds'].items()}
        assert meta['binary_paths']==binaries and meta['runtime_environment']=={'NDLOCR_INTRA_THREADS':'1','NDLOCR_PARSEQ_PARALLELISM':'1','LD_LIBRARY_PATH':str(stage/'runtime')}
        planned=full.worker_commands(bundle.index['assets'],binaries,selected,kind,work,work.parent/'evidence',models)
        assert meta['planned_commands']==planned
        receipts=[json.loads(raw) for name,raw in worker.items() if name.startswith('evidence/commands/') and name.endswith('.json')]
        assert len(receipts)==len(planned)==len(meta['commands']) and all(r['exit_code']==0 for r in receipts)
        assert Counter(json.dumps(r,sort_keys=True) for r in receipts)==Counter(json.dumps(r,sort_keys=True) for r in meta['commands'])
        for command in planned:
            receipt=json.loads(worker['evidence/commands/'+command['label']+'.json']);assert receipt['argv']==command['argv'] and receipt['exit_code']==0
        for variant in ['baseline','candidate']:
            if kind.startswith('page'):
                for setting in ['eval','default']:
                    prefix=f'evidence/page/{variant}/runtime{kind[4:]}_{setting}/';rows=json.loads(worker[prefix+'results.json'])
                    assert [r['id'] for r in rows]==[r['id'] for r in selected]
                    for row,source in zip(rows,selected,strict=True):
                        assert worker[prefix+row['id']+'.txt'].decode()==row['text']=='\n'.join(r['text'] for r in row['lines']) and type(row['page_index']) is int and row['page_index']==0
                        for line in row['lines']:
                            c=line['confidence'];box=line['bbox']
                            assert type(c) in [int,float] and math.isfinite(c) and 0<=c<=1
                            assert len(box)==4 and all(type(x) is int for x in box) and 0<=box[0]<box[2]<=source['width'] and 0<=box[1]<box[3]<=source['height']
                    merge(page.setdefault((kind,setting,variant),{}),{r['id']:r for r in rows})
                    merge(predictions[kind].setdefault(variant+'_'+setting,{}),{r['id']:r['text'] for r in rows})
            else:
                values={}
                for source in selected:
                    ident=source['id'];body=worker[f'evidence/cli/{variant}/{ident}.txt'];assert worker[f'evidence/commands/{variant}-{kind}-{ident}.stdout']==body+b'\n';values[ident]=body.decode()
                merge(predictions[kind].setdefault(variant,{}),values)
    def canonical(row):
        return {'text':row['text'],'page_index':row['page_index'],'lines':[{'bbox':r['bbox'],'text':r['text'],'confidence_bits':struct.unpack('!I',struct.pack('!f',r['confidence']))[0]} for r in row['lines']]}
    changes=[]
    for kind in full.KINDS[:3]:
        for setting in ['eval','default']:
            for source in bundle.index['physical_inputs']:
                ident=source['id'];a=canonical(page[kind,setting,'baseline'][ident]);b=canonical(page[kind,setting,'candidate'][ident])
                if a!=b:changes.append({'kind':kind,'setting':setting,'id':ident,'before':a,'after':b})
    comparison=members(next(raw for name,raw in data.items() if name.startswith('artifacts/') and '-comparison-' in name))
    summary=json.loads(comparison['summary.json']);assert summary['complete'] and summary['bundle_sha256']==bundle_sha and summary['build_receipt_sha256']==build_sha
    scores=bundle.index['scoring_inputs'];verified=0;pairs=[];regressions=[];gains=[]
    for score in scores:assert comparison['metrics/truth/'+score['id']+'.txt']==bundle.read_truth(score)
    for kind,systems in predictions.items():
        mapped={key:matrix.map_predictions(scores,values) for key,values in systems.items()}
        for key,values in mapped.items():
            for ident,text in values.items():assert comparison[f'predictions/{kind}/{key}/{ident}.txt'].decode()==text
        for mode in ['none','width','strict']:
            rows=list(csv.DictReader(io.StringIO(comparison[f'metrics/{kind}/{mode}.csv'].decode())))
            assert len(rows)==len(scores)*len(systems) and {(r['system'],r['file']) for r in rows}=={(k,s['id']+'.txt') for k in systems for s in scores}
            for row in rows:
                expected=metric_values(comparison['metrics/truth/'+row['file']].decode(),mapped[row['system']][row['file'][:-4]],mode)
                assert tuple(int(row[k]) for k in ['char_errors','char_total','word_errors','word_total'])==expected;verified+=1
            indexed={(r['system'],r['file']):r for r in rows}
            for setting in (['eval','default'] if kind.startswith('page') else [None]):
                akey='baseline'+('_'+setting if setting else '');bkey='candidate'+('_'+setting if setting else '')
                for score in scores:
                    name=score['id']+'.txt';a=indexed[akey,name];b=indexed[bkey,name]
                    entry={'kind':kind,'setting':setting,'normalization':mode,'scoring_id':score['id'],'physical_id':score['physical_id'],'origin':score['origin'],'before':a,'after':b,'body_unchanged':systems[akey][score['physical_id']]==systems[bkey][score['physical_id']]};pairs.append(entry)
                    if any(int(b[k])>int(a[k]) for k in ['char_errors','word_errors']):regressions.append(entry)
                    if mode=='none' and int(b['char_errors'])<int(a['char_errors']) and int(b['word_errors'])<=int(a['word_errors']):gains.append(entry)
    full.require_complete_metric_count(verified,bundle.plan)
    for name,values in [('all-input-pairs',pairs),('regressions',regressions),('raw-known-gains',gains),('page-changes',changes)]:assert json.loads(comparison[name+'.json'])==values
    assert summary['image_results']==image_results and summary['verified_CSV_rows']==verified and summary['scoring_comparison_pairs']==len(pairs)
    assert summary['regression_pairs']==len(regressions) and summary['page_body_bbox_confidence_changes']==len(changes) and summary['raw_gain_scoring_pairs']==len(gains)
    assert summary['full_gate_pass']==(not regressions and not changes)
    return {'complete_workers':len(metas),'image_results':image_results,'verified_CSV_rows':verified,'scoring_comparison_pairs':len(pairs),'regression_pairs':len(regressions),'page_body_bbox_confidence_changes':len(changes),'raw_known_gain_pairs':len(gains),'full_gate_pass':summary['full_gate_pass'],'new_model_calls_in_replay':0,'native_bytes_in_public_packet':False}


if __name__=='__main__':
    import argparse
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('folder',type=Path);args=parser.parse_args();print(json.dumps(verify(args.folder),indent=2))
