"""Replay every completed R native result and official score without inference."""
import csv
import io
import json
from pathlib import Path
from moderate_prefix_native_metrics import pair_decision
from verify_additional_real_baseline import _packet
from verify_moderate_prefix_native_before import verify as verify_before


def verify(folder):
    folder=Path(folder);verify_before(folder)
    bi,before=_packet(folder,'before-native-v1.index.json');index,data=_packet(folder,'completed-native-v1.index.json');ni,controls=_packet(folder,'nn0-v1.index.json')
    assert index['before_native_packet_sha256']==bi['sha256']
    pre=json.loads(before['evidence/native-before-NN-v1.json']);job=json.loads(data['evidence/native-job-v1.json']);scope=pre['scope'];rows=json.loads(before['evidence/new-manifest-v2.json'])['inputs'];manifest=json.loads(before['evidence/native-manifest-v1.json'])
    assert job['complete'] and all(job[k]==v for k,v in pre.items() if k not in ['complete','commands'])
    assert len(job['commands'])==6 and {c['label'] for c in job['commands']}=={c['label'] for c in pre['commands_planned']}
    results={};outer=0
    for command in pre['commands_planned']:
        label=command['label'];receipt=json.loads(data['native/'+label+'-command.json']);assert receipt in job['commands'] and all(receipt[k]==v for k,v in command.items()) and receipt['exit_code']==0 and receipt['frozen_files_after_unchanged']
        assert b'1 passed; 0 failed;' in data['native/'+label+'.stdout']
        loaded=json.loads(data['native/'+label+'/models-loaded.json']);model=next(m for m in manifest['models'] if m['label']==command['model']);assert loaded['model']==model and loaded['model_loads']==3 and loaded['input_dimensions']==[{'30':256,'50':384,'100':768}[model['label']],24]
        values=json.loads(data['native/'+label+'/results.json']);assert [r['id'] for r in values]==scope['input_ids'] and len(values)==72
        for value in values:
            assert set(value)=={'id','label',*scope['routes']} and value['label']==model['label']
            assert value['fragment_auto']==value['fragment_batch_auto']==value['fragment_exclusive_batch_auto']
            assert json.loads(data['native/'+label+'/'+value['id']+'.json'])==value
        results[command['variant'],model['label']]={r['id']:r for r in values}
        summary=json.loads(data['native/'+label+'/summary.json']);assert summary['inputs']==72 and summary['label']==model['label'] and summary['image_results']==576 and summary['outer_image_API_calls']==434 and summary['API_errors']==0 and summary['batch_error'] is summary['exclusive_batch_error'] is None and summary['fragment_single_batch_exclusive_equal'] and summary['internal_session_run_calls'] is None;outer+=summary['outer_image_API_calls']
    comparisons=[];systems={}
    for row in rows:assert data['score/truth/'+row['id']+'.txt']==controls['inputs/'+row['id']+'.txt']
    for variant in scope['variants']:
        for model in scope['models']:
            for route in scope['routes']:
                system=variant+'_'+model+'_'+route;systems[system]={}
                for row in rows:
                    ident=row['id'];text=results[variant,model][ident][route]['text'];assert data['score/'+system+'/'+ident+'.txt'].decode()==text;systems[system][ident]=text
    for model in scope['models']:
        for row in rows:
            ident=row['id'];truth=data['score/truth/'+ident+'.txt'].decode()
            for route in scope['routes']:
                a=results['baseline',model][ident][route];b=results['candidate',model][ident][route];comparisons.append({'id':ident,'model':model,'route':route,'cohort':row['cohort'],'before':a,'after':b,**pair_decision(a,b,truth,route,row['cohort'])})
    verified=0;metric_commands=json.loads(data['score/metric-commands.json']);assert len(metric_commands)==3
    for mode,command in zip(['none','width','strict'],metric_commands,strict=True):
        output=Path(command[-1]).parent;wanted=[command[0],str(Path(pre['commands_planned'][0]['env']['NDLOCR_UNCERTAIN_MATRIX_MANIFEST']).parents[2]/'tools/eval_ocr_metrics.py'),'--truth-dir',str(output/'truth')]
        for system in systems:wanted += ['--system',system+'='+str(output/system)]
        wanted+=['--normalize',mode,'--require-all','--output-csv',str(output/(mode+'.csv'))];assert command==wanted
        csv_rows=list(csv.DictReader(io.StringIO(data['score/'+mode+'.csv'].decode())));assert len(csv_rows)==3456 and {(r['system'],r['file']) for r in csv_rows}=={(s,r['id']+'.txt') for s in systems for r in rows};indexed={(r['system'],r['file']):r for r in csv_rows}
        for comparison in comparisons:
            for variant in scope['variants']:
                row=indexed[variant+'_'+comparison['model']+'_'+comparison['route'],comparison['id']+'.txt'];expected=comparison['metrics'][mode]['before' if variant=='baseline' else 'after'];assert tuple(int(row[k]) for k in ['char_errors','char_total','word_errors','word_total'])==tuple(expected);verified+=1
    regressions=[r for r in comparisons if r['regression']];protected=[r for r in comparisons if not r['protected_unchanged']];gains=[r for r in comparisons if r['raw_body_gain']];changed=[r for r in comparisons if not r['body_unchanged']]
    for name,value in [('comparisons',comparisons),('regressions',regressions),('protected-changes',protected),('body-gains',gains),('changed',changed)]:assert json.loads(data['score/'+name+'.json'])==json.loads(json.dumps(value))
    expected={'complete':True,'image_results':3456,'verified_CSV_rows':verified,'raw_comparison_pairs':len(comparisons),'normalized_comparison_pairs':len(comparisons)*3,'regression_pairs':len(regressions),'protected_changes':len(protected),'new_body_fragment_auto_gains':len(gains),'raw_changed_pairs':len(changed),'new72_gate_pass':not regressions and not protected and bool(gains),'internal_session_run_calls':None,'NN_calls_in_scoring':0,'frozen_files_after_unchanged':True};assert expected==json.loads(data['score/summary.json'])
    assert verified==10368 and len(comparisons)==1728 and outer==2604 and index['new72_gate_pass']==expected['new72_gate_pass'] and index['NN_calls_in_replay']==0
    return {k:expected[k] for k in ['image_results','verified_CSV_rows','raw_comparison_pairs','normalized_comparison_pairs','regression_pairs','protected_changes','new_body_fragment_auto_gains','new72_gate_pass']}|{'meanbit_changes':sum(not r['confidence_bits_unchanged'] for r in comparisons),'NN_calls_in_replay':0}


if __name__=='__main__':
    print(json.dumps(verify(Path(__file__).resolve().parents[1]/'docs/benchmarks/vertical-moderate-prefix'),indent=2))
