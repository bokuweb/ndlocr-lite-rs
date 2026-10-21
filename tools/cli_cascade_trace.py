"""Run isolated, known-input CLI stage observations, not an inference policy."""
from vertical_page_matrix import cli_command


def trace_command(binary, image, models, assets, config, body, missing_optional):
    if config not in ('default', 'model30', 'model50'):
        raise ValueError('unknown fixed trace configuration')
    command = cli_command(binary, image, models, assets, 'deim', body)
    if config == 'default':
        return command
    command[command.index('--model') + 1] = str(models / assets[config[5:]]['name'])
    for name in ['--model30', '--model50']:
        command[command.index(name) + 1] = str(missing_optional)
    return command + ['--enable-cascade', 'false']


def trace_inputs_changed(paths):
    return any(path in {'.github/workflows/cli-cascade-trace.yml','tools/cli_cascade_trace.py',
                        'tests/test_cli_cascade_trace.py'}
               or path in {'docs/benchmarks/vertical-cli-cascade-trace/'+name
                           for name in ['bundle.json','data.zip','sources.zip']} for path in paths)


def run(bundle, output, models):
    import csv
    import json
    import os
    import platform
    import subprocess
    import sys
    from pathlib import Path
    from PIL import Image
    import vertical_page_matrix as matrix
    from verify_low_stop_evidence import metric_values
    repository=Path(__file__).resolve().parents[1]
    output=Path(output).resolve();output.mkdir(parents=True,exist_ok=False)
    evidence=output/'evidence';evidence.mkdir();work=output/'work';bundle.materialize(work)
    models=Path(models).resolve()
    assert sys.platform.startswith('linux'), 'This frozen trace protocol requires Linux'
    assert 'NDLOCR_CLI_STAGE_TRACE_DIR' not in os.environ
    assets=bundle.index['assets']
    for asset in assets.values():assert matrix.sha((models/asset['name']).read_bytes())==asset['sha256']
    meta={'complete':False,'bundle_sha256':matrix.sha((bundle.folder/'bundle.json').read_bytes()),'profile':'release',
          'platform':platform.platform(),'rustc':subprocess.check_output(['rustc','-Vv'],text=True),'builds':{},'commands':[],
          'assets':assets,'internal_forward_calls':None,'model_session_loads':None,'target_CLI_processes':18,'target_CSV_rows':54}
    matrix.write_json(evidence/'job.json',meta)
    env={'NDLOCR_INTRA_THREADS':'1','NDLOCR_PARSEQ_PARALLELISM':'1'};meta['runtime_environment']=env
    try:
        matrix.run_command(['cargo','test','--locked','--manifest-path',work/'candidate/Cargo.toml','--lib','app::cascade_trace'],
                           'observer-pure-tests',evidence,meta,{'CARGO_TARGET_DIR':str(work/'target'),'CARGO_PROFILE_TEST_DEBUG':'0','CARGO_BUILD_JOBS':'2'})
        binaries={}
        for variant in ['baseline','candidate']:
            binaries[variant]=matrix.build_variant(bundle,variant,work,evidence,work/'target',models,repository,meta)
        assert binaries['baseline']['cli']['sha256']!=binaries['candidate']['cli']['sha256']
        matrix.write_json(evidence/'before-NN.json',meta)
        predictions={};observed_crops=0;batch_calls=0;successful_CLI_processes=0
        missing=work/'not-provided.onnx';assert not missing.exists()
        for config in ['default','model30','model50']:
            for variant in ['baseline','candidate']:
                key=variant+'_'+config;directory=evidence/'predictions'/key;directory.mkdir(parents=True);predictions[key]={}
                for row in bundle.index['physical_inputs']:
                    ident=row['id'];label=key+'_'+ident;body=directory/(ident+'.txt');trace=evidence/'trace'/config/ident
                    local_env=dict(env)
                    if variant=='candidate':
                        trace.parent.mkdir(parents=True,exist_ok=True);assert not trace.exists()
                        local_env['NDLOCR_CLI_STAGE_TRACE_DIR']=str(trace)
                    argv=trace_command(binaries[variant]['cli']['path'],work/row['image'],models,assets,config,body,missing)
                    matrix.run_command(argv,label,evidence,meta,local_env)
                    predictions[key][ident]=matrix.read_cli_result(evidence/'commands'/(label+'.stdout'),body)
                    successful_CLI_processes+=1
                    if variant=='candidate':
                        other='baseline_'+config+'_'+ident
                        for suffix in ['stdout','stderr']:
                            assert (evidence/'commands'/(other+'.'+suffix)).read_bytes()==(evidence/'commands'/(label+'.'+suffix)).read_bytes()
                        assert predictions[key][ident]==predictions['baseline_'+config][ident]
                        image=Image.open(work/row['image']).convert('RGB');events=sorted(trace.glob('*.json'));assert events
                        phases=[]
                        for event_path in events:
                            event=json.loads(event_path.read_text());phases.append(event['phase'])
                            if event['kind']=='batch' and event['items']:batch_calls+=1
                            assert [r['index'] for r in event['results']]==[i['index'] for i in event['items']]
                            for item in event['items']:
                                x0,y0,x1,y1=item['bbox'];assert 0<=x0<x1<=image.width and 0<=y0<y1<=image.height
                                crop=item['crop'];assert [crop['width'],crop['height']]==[x1-x0,y1-y0]
                                assert (trace/matrix.safe_member(crop['file'])).read_bytes()==image.crop((x0,y0,x1,y1)).tobytes()
                                observed_crops+=1
                        assert 'initial-final' in phases and 'page-batch-final' in phases
                        assert ('cascade-off' in phases)==(config!='default')
                    meta['successful_CLI_processes']=successful_CLI_processes;matrix.write_json(evidence/'job.json',meta)
        truth=evidence/'metrics/truth';truth.mkdir(parents=True)
        for row in bundle.index['scoring_inputs']:(truth/(row['id']+'.txt')).write_bytes(bundle.read('data.zip',row['truth']))
        verified=0
        for mode in ['none','width','strict']:
            table=evidence/'metrics'/(mode+'.csv');argv=[sys.executable,str(repository/'tools/eval_ocr_metrics.py'),'--truth-dir',str(truth)]
            for key in predictions:argv+=['--system',key+'='+str(evidence/'predictions'/key)]
            argv+=['--normalize',mode,'--require-all','--output-csv',str(table)];matrix.run_command(argv,'metrics-'+mode,evidence,meta)
            rows=list(csv.DictReader(table.open()));assert len(rows)==18 and len({(r['system'],r['file']) for r in rows})==18
            for row in rows:
                expected=metric_values((truth/row['file']).read_text(),(evidence/'predictions'/row['system']/row['file']).read_text(),mode)
                assert tuple(int(row[k]) for k in ['char_errors','char_total','word_errors','word_total'])==expected;verified+=1
        for variant in binaries:
            for name,digest in bundle.index['source_files'][variant].items():assert matrix.sha((work/variant/name).read_bytes())==digest
            for binary in binaries[variant].values():assert matrix.sha(Path(binary['path']).read_bytes())==binary['sha256']
        for asset in assets.values():assert matrix.sha((models/asset['name']).read_bytes())==asset['sha256']
        for row in bundle.index['physical_inputs']:assert matrix.sha((work/row['image']).read_bytes())==row['image_sha256']
        assert successful_CLI_processes==18 and verified==54
        meta.update(complete=True,frozen_materials_after_unchanged=True,verified_CSV_rows=verified,image_results=18,
                    recorded_crop_instances_verified=observed_crops,observed_nonempty_pool_batch_calls=batch_calls,
                    observer_additional_NN_requests=0,all_normal_trace_outputs_equal=True,adoption_comparison=False)
    except Exception as error:
        meta['error']=repr(error);raise
    finally:matrix.write_json(evidence/'job.json',meta)
    return meta


if __name__=='__main__':
    import argparse
    import json
    from pathlib import Path
    import subprocess
    from vertical_page_matrix import Bundle
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--changes-from')
    parser.add_argument('--validate',action='store_true')
    parser.add_argument('--bundle',type=Path,default=Path(__file__).resolve().parents[1]/'docs/benchmarks/vertical-cli-cascade-trace')
    parser.add_argument('--output',type=Path)
    parser.add_argument('--models',type=Path,default=Path('models'))
    args=parser.parse_args()
    if args.changes_from:
        paths=subprocess.check_output(['git','diff','--name-only',args.changes_from,'HEAD'],text=True).splitlines()
        print('run='+str(trace_inputs_changed(paths)).lower())
    else:
        bundle=Bundle(args.bundle)
        if args.validate:print('frozen trace bundle verified; NN calls=0')
        elif args.output:run(bundle,args.output,args.models)
        else:parser.error('--validate or --output is required')
