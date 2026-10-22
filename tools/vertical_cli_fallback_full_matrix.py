"""Frozen complete FVC matrix contracts and execution."""
import re
from vertical_page_matrix import cli_command
from vertical_page_shards import partition_inputs

KINDS=('page30','page50','page100','cli_deim_on','cli_deim_off','cli_bands_on','cli_bands_off')
HARNESS_FILES=('tools/vertical_cli_fallback_full_matrix.py','tools/vertical_page_matrix.py','tools/vertical_page_shards.py',
               'tools/verify_vertical_cli_fallback.py','tools/verify_low_stop_evidence.py','tools/eval_ocr_metrics.py',
               'docs/benchmarks/vertical-partition-agreement/fast_distance.py')


def input_relative_path(row):
    from pathlib import Path
    ident=row['id'];suffix=Path(row['image']).suffix.lower()
    if not re.fullmatch(r'[A-Za-z0-9_-]+',ident) or suffix not in {'.png','.jpg','.jpeg'}:
        raise ValueError('invalid frozen image name or format')
    return Path('inputs')/(ident+suffix)


def merge_controls(groups):
    images=[];scores=[]
    valid=lambda value:isinstance(value,str) and re.fullmatch(r'[A-Za-z0-9_-]+',value)
    for group in groups:
        physical=group.get('inputs',[]);scoring=group.get('scoring_inputs',[]);origin=group.get('origin')
        ids=[r.get('id') for r in physical];aliases=[r.get('id') for r in scoring]
        if (not valid(origin) or not ids or not aliases or any(not valid(v) for v in ids+aliases)
                or len(set(ids))!=len(ids) or len(set(aliases))!=len(aliases)
                or {r.get('physical_id') for r in scoring}!=set(ids)):
            raise ValueError('invalid physical/scoring group')
        images.extend(dict(r,origin=origin) for r in physical)
        scores.extend(dict(r,origin=origin) for r in scoring)
    if (not images or len({r['id'] for r in images})!=len(images)
            or len({r['id'] for r in scores})!=len(scores)):
        raise ValueError('missing or duplicate IDs across groups')
    return images,scores


def full_plan(inputs,scoring,total=8):
    partition_inputs(inputs,0,total)
    return {'kinds':list(KINDS),'jobs':[{'kind':kind,'index':i,'total':total,
               'physical_ids':[r['id'] for r in partition_inputs(inputs,i,total)]} for kind in KINDS for i in range(total)],
            'physical_inputs':len(inputs),'scoring_inputs':len(scoring),
            'expected_image_results':len(inputs)*20,'expected_CSV_rows':len(scoring)*60}


def full_cli_command(binary,image,models,assets,kind,body):
    if kind not in KINDS[3:]:raise ValueError('unknown full CLI kind')
    _,mode,switch=kind.split('_')
    return cli_command(binary,image,models,assets,mode,body)+['--enable-cascade',str(switch=='on').lower()]


def validate_complete_jobs(metas,plan,bundle_sha,build_sha):
    expected={(j['kind'],j['index']):j for j in plan['jobs']}
    if len(metas)!=len(expected):raise ValueError('missing or duplicate job')
    seen=set();images=0
    for meta in metas:
        shard=meta.get('shard',{});key=(meta.get('kind'),shard.get('index'))
        if key not in expected or key in seen:raise ValueError('unknown or duplicate shard')
        wanted=expected[key]
        if (meta.get('complete') is not True or meta.get('frozen_materials_after_unchanged') is not True
                or meta.get('bundle_sha256')!=bundle_sha or meta.get('build_receipt_sha256')!=build_sha
                or shard!={k:wanted[k] for k in ['index','total','physical_ids']}):
            raise ValueError('partial or mismatched source/data shard')
        count=len(wanted['physical_ids'])*(4 if wanted['kind'].startswith('page') else 2)
        if meta.get('image_results')!=count:raise ValueError('incomplete image results')
        seen.add(key);images+=count
    if images!=plan['expected_image_results']:raise ValueError('global count mismatch')
    return images


def require_complete_metric_count(actual,plan):
    expected=plan.get('expected_CSV_rows')
    if not isinstance(expected,int) or expected<=0 or actual!=expected:
        raise ValueError('official metric rows do not cover the complete frozen plan')


class FullBundle:
    def __init__(self,folder):
        import io
        import json
        from pathlib import Path
        import zipfile
        from vertical_page_matrix import Bundle,sha
        from verify_vertical_cli_fallback import packet,verify
        self.folder=Path(folder)
        self.index,self.extra=packet(self.folder,'bundle.json')
        assert sha((self.folder/'protocol-before-nn.md').read_bytes())==self.index['protocol_sha256']
        self.base=Bundle(self.folder.parent.parent/'vertical-page-fallback')
        assert sha((self.base.folder/'bundle.json').read_bytes())==self.index['base_bundle_sha256']
        adapter_index,adapter=packet(self.folder.parent,'adapter-before-nn-v1.json')
        assert adapter_index['sha256']==self.index['adapter_packet_sha256']
        result_index=json.loads((self.folder.parent/'new72-cli-matrix-v1.json').read_text())
        assert result_index['sha256']==self.index['new72_result_packet_sha256']
        assert verify(self.folder.parent)['new72_gate_pass']
        self.sources={}
        for variant,expected in self.index['source_files'].items():
            name='tmp/vertical-cli-fallback/adapter-final-'+variant+'-source.zip'
            with zipfile.ZipFile(io.BytesIO(adapter[name])) as z:
                assert len(z.namelist())==len(set(z.namelist())) and set(z.namelist())==set(expected)
                values={n:z.read(n) for n in expected}
            assert all(sha(values[n])==h for n,h in expected.items())
            self.sources[variant]=values
        original=self.base.index['source_files']['candidate'];baseline=self.index['source_files']['baseline']
        assert all(baseline[n]==h for n,h in original.items())
        assert set(baseline)-set(original)=={'tests/vertical_fallback_cli_tests.rs'}
        self.baseline_FVB_bridge=True
        physical=self.index['physical_inputs'];scoring=self.index['scoring_inputs'];expected=self.index['expected']
        assert len(physical)==expected['physical_inputs']==1239 and len({r['id'] for r in physical})==1239
        assert len(scoring)==expected['scoring_inputs']==1245 and len({r['id'] for r in scoring})==1245
        assert {r['physical_id'] for r in scoring}=={r['id'] for r in physical}
        assert len({(r['width'],r['height'],r['rgb_sha256']) for r in physical})==expected['unique_RGB']==1238
        for row in physical:assert sha(self.read_image(row))==row['image_sha256']
        for row in scoring:assert sha(self.read_truth(row))==row['truth_sha256']
        self.plan=full_plan(physical,scoring,self.index['shards'])
        assert self.plan==json.loads((self.folder/'plan.json').read_text())
        assert self.plan['expected_image_results']==expected['image_results']==24780
        assert self.plan['expected_CSV_rows']==expected['CSV_rows']==74700
        assert self.index['assets']==self.base.index['assets']

    def read_image(self,row):
        return self.base.read('data.zip',row['image']) if row['origin']=='base' else self.extra[row['image']]

    def read_truth(self,row):
        return self.base.read('data.zip',row['truth']) if row['origin']=='base' else self.extra[row['truth']]

    def materialize_sources(self,work):
        from vertical_page_matrix import safe_member
        for variant,files in self.sources.items():
            for name,data in files.items():
                path=work/variant/safe_member(name);path.parent.mkdir(parents=True,exist_ok=True);path.write_bytes(data)

    def materialize_inputs(self,work,selected):
        import re
        directory=work/'inputs';directory.mkdir(parents=True,exist_ok=False)
        rows=[]
        for row in selected:
            assert re.fullmatch(r'[A-Za-z0-9_-]+',row['id'])
            path=work/input_relative_path(row);path.write_bytes(self.read_image(row));rows.append({'id':row['id'],'image':str(path)})
        return {'inputs':rows}


def build_pair(bundle,output):
    import json
    import platform
    from pathlib import Path
    import shutil
    import subprocess
    import sys
    import vertical_page_matrix as matrix
    if not sys.platform.startswith('linux'):raise ValueError('full frozen gate requires Linux')
    output=Path(output).resolve();output.mkdir(parents=True,exist_ok=False);work=output/'work';evidence=output/'evidence';evidence.mkdir();stage=output/'stage';stage.mkdir()
    bundle.materialize_sources(work)
    meta={'complete':False,'bundle_sha256':matrix.sha((bundle.folder/'bundle.json').read_bytes()),'profile':'release','platform':platform.platform(),
          'rustc':subprocess.check_output(['rustc','-Vv'],text=True),'producer_sha256':matrix.sha(Path(__file__).read_bytes()),'builds':{},'commands':[],
          'NN_calls':0,'model_loads':0,'baseline_FVB_bridge':bundle.baseline_FVB_bridge}
    repository=Path(__file__).resolve().parents[1]
    meta['harness_source_files']={n:matrix.sha((repository/n).read_bytes()) for n in HARNESS_FILES}
    matrix.write_json(evidence/'job.json',meta)
    try:
        for variant in ['baseline','candidate']:
            source=work/variant;expected=bundle.index['source_files'][variant]
            assert all(matrix.sha((source/n).read_bytes())==h for n,h in expected.items())
            for p in source.rglob('*.rs'):p.touch()
            argv=['cargo','build','--locked','--release','--manifest-path',source/'Cargo.toml','--features','onnx','--bin','ndlocr-lite-rs','--example','capture_page_matrix','--message-format=json']
            matrix.run_command(argv,variant+'-build',evidence,meta,{'CARGO_TARGET_DIR':str(work/'target'),'CARGO_BUILD_JOBS':'2','CARGO_PROFILE_RELEASE_DEBUG':'0'})
            raw=(evidence/'commands'/(variant+'-build.stdout')).read_bytes();events=[json.loads(s) for s in raw.decode().splitlines() if s.strip()]
            artifacts=matrix.check_build_artifacts(events,source/'Cargo.toml');binaries={}
            for key in ['cli','page']:
                original=Path(artifacts[key]['executable']);name='binaries/'+variant+'-'+key;destination=stage/name;destination.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(original,destination);destination.chmod(0o755)
                assert destination.read_bytes()==original.read_bytes()
                binaries[key]={'file':name,'sha256':matrix.sha(destination.read_bytes()),'bytes':destination.stat().st_size,'compiler_executable':str(original)}
            assert all(matrix.sha((source/n).read_bytes())==h for n,h in expected.items())
            meta['builds'][variant]={'source_manifest':str(source/'Cargo.toml'),'source_files':expected,'artifacts':artifacts,'build_stdout_sha256':matrix.sha(raw),'binaries':binaries}
            matrix.write_json(evidence/'job.json',meta)
        assert meta['builds']['baseline']['binaries']['cli']['sha256']!=meta['builds']['candidate']['binaries']['cli']['sha256']
        runtime={}
        for path in (work/'target').rglob('libonnxruntime*.so*'):
            if path.is_file():
                raw=path.read_bytes()
                if path.name in runtime:assert runtime[path.name]==raw
                runtime[path.name]=raw
        for name,raw in runtime.items():
            path=stage/'runtime'/name;path.parent.mkdir(parents=True,exist_ok=True);path.write_bytes(raw);path.chmod(0o755)
        (stage/'runtime').mkdir(exist_ok=True)
        for variant,build in meta['builds'].items():
            for key,value in build['binaries'].items():
                label=variant+'-'+key+'-ldd'
                matrix.run_command(['ldd',stage/value['file']],label,evidence,meta,{'LD_LIBRARY_PATH':str(stage/'runtime')})
                assert 'not found' not in (evidence/'commands'/(label+'.stdout')).read_text()
        shutil.copytree(evidence/'commands',stage/'commands')
        for name,digest in meta['harness_source_files'].items():
            raw=(repository/name).read_bytes();assert matrix.sha(raw)==digest
            path=stage/'harness'/name;path.parent.mkdir(parents=True,exist_ok=True);path.write_bytes(raw)
        assert matrix.sha(Path(__file__).read_bytes())==meta['producer_sha256']
        meta.update(complete=True,source_files_after_unchanged=True,runtime_libraries={n:matrix.sha(raw) for n,raw in runtime.items()},
                    stage_files={p.relative_to(stage).as_posix():{'bytes':p.stat().st_size,'sha256':matrix.sha(p.read_bytes())} for p in stage.rglob('*') if p.is_file()})
        matrix.write_json(stage/'build.json',meta)
    except Exception as error:meta['error']=repr(error);raise
    finally:matrix.write_json(evidence/'job.json',meta)
    return meta


def validate_stage(bundle,stage):
    import json
    from pathlib import Path
    import vertical_page_matrix as matrix
    stage=Path(stage);meta=json.loads((stage/'build.json').read_text())
    assert meta['complete'] and meta['source_files_after_unchanged'] and meta['baseline_FVB_bridge']
    assert meta['bundle_sha256']==matrix.sha((bundle.folder/'bundle.json').read_bytes())
    assert meta['NN_calls']==0 and meta['model_loads']==0
    for name,digest in meta['harness_source_files'].items():assert matrix.sha((stage/'harness'/matrix.safe_member(name)).read_bytes())==digest
    assert {p.relative_to(stage).as_posix() for p in stage.rglob('*') if p.is_file()}==set(meta['stage_files'])|{'build.json'}
    for n,h in meta['stage_files'].items():
        raw=(stage/matrix.safe_member(n)).read_bytes();assert len(raw)==h['bytes'] and matrix.sha(raw)==h['sha256']
    for variant,build in meta['builds'].items():
        assert build['source_files']==bundle.index['source_files'][variant]
        raw=(stage/'commands'/(variant+'-build.stdout')).read_bytes();assert matrix.sha(raw)==build['build_stdout_sha256']
        events=[json.loads(line) for line in raw.decode().splitlines() if line.strip()]
        assert matrix.check_build_artifacts(events,build['source_manifest'])==build['artifacts']
        for value in build['binaries'].values():
            raw=(stage/matrix.safe_member(value['file'])).read_bytes();assert len(raw)==value['bytes'] and matrix.sha(raw)==value['sha256']
    assert meta['builds']['baseline']['binaries']['cli']['sha256']!=meta['builds']['candidate']['binaries']['cli']['sha256']
    return meta


def worker_commands(assets,binaries,selected,kind,work,evidence,models):
    if kind not in KINDS:raise ValueError('unknown kind')
    commands=[]
    for variant in ['baseline','candidate']:
        if kind.startswith('page'):
            model=kind[4:];directory=evidence/'page'/variant
            argv=[binaries[variant]['page'],'--manifest',work/'NN-images-only.json','--det-model',models/assets['deim']['name'],
                  '--parseq-model',models/assets[model]['name'],'--charset',models/assets['charset']['name'],
                  '--label','runtime'+model,'--output-dir',directory]
            commands.append({'variant':variant,'label':variant+'-'+kind,'argv':[str(x) for x in argv],'target':str(directory)})
        else:
            for row in selected:
                ident=row['id'];body=evidence/'cli'/variant/(ident+'.txt')
                argv=full_cli_command(binaries[variant]['cli'],work/input_relative_path(row),models,assets,kind,body)
                commands.append({'variant':variant,'id':ident,'label':variant+'-'+kind+'-'+ident,'argv':[str(x) for x in argv],'target':str(body)})
    return commands


def run_shard(bundle,stage,kind,index,total,output,models):
    import json
    import os
    from pathlib import Path
    import platform
    import shutil
    import sys
    import vertical_page_matrix as matrix
    if not sys.platform.startswith('linux') or kind not in KINDS:raise ValueError('invalid full worker platform/kind')
    if total!=bundle.index['shards']:raise ValueError('worker partition differs from frozen plan')
    stage=Path(stage).resolve();build=validate_stage(bundle,stage);models=Path(models).resolve()
    assert matrix.sha(Path(__file__).read_bytes())==build['producer_sha256']
    output=Path(output).resolve();output.mkdir(parents=True,exist_ok=False);work=output/'work';evidence=output/'evidence';evidence.mkdir()
    selected=partition_inputs(bundle.index['physical_inputs'],index,total);ids={r['id'] for r in selected};scoring=[r for r in bundle.index['scoring_inputs'] if r['physical_id'] in ids]
    shard={'index':index,'total':total,'physical_ids':[r['id'] for r in selected]}
    manifest=bundle.materialize_inputs(work,selected);manifest_path=work/'NN-images-only.json';matrix.write_json(manifest_path,manifest)
    meta={'complete':False,'kind':kind,'shard':shard,'bundle_sha256':matrix.sha((bundle.folder/'bundle.json').read_bytes()),
          'build_receipt_sha256':matrix.sha((stage/'build.json').read_bytes()),'producer_sha256':matrix.sha(Path(__file__).read_bytes()),
          'global_physical_inputs':len(bundle.index['physical_inputs']),'global_scoring_inputs':len(bundle.index['scoring_inputs']),
          'physical_inputs':len(selected),'scoring_inputs':len(scoring),'commands':[],'assets':bundle.index['assets'],'models_dir':str(models),
          'manifest':manifest,'manifest_sha256':matrix.sha(manifest_path.read_bytes()),'platform':platform.platform(),'profile':'release',
          'internal_forward_calls':None,'CLI_model_session_loads':None,'successful_PageRecognizer_loads':0,'successful_Page_model_session_loads':0,
          'successful_Page_API_results':0,'successful_CLI_processes':0,'binary_paths':{},'stage_directory':str(stage)}
    env={'NDLOCR_INTRA_THREADS':'1','NDLOCR_PARSEQ_PARALLELISM':'1','LD_LIBRARY_PATH':str(stage/'runtime')};meta['runtime_environment']=env
    def check():
        validate_stage(bundle,stage)
        repository=Path(__file__).resolve().parents[1]
        for name,digest in build['harness_source_files'].items():assert matrix.sha((repository/name).read_bytes())==digest
        assert matrix.sha((stage/'build.json').read_bytes())==meta['build_receipt_sha256']
        assert matrix.sha((bundle.folder/'bundle.json').read_bytes())==meta['bundle_sha256']
        assert matrix.sha(Path(__file__).read_bytes())==meta['producer_sha256']
        for a in bundle.index['assets'].values():assert matrix.sha((models/a['name']).read_bytes())==a['sha256']
        for row in selected:assert matrix.sha((work/input_relative_path(row)).read_bytes())==row['image_sha256']
        for row in scoring:assert matrix.sha(bundle.read_truth(row))==row['truth_sha256']
        assert matrix.sha(manifest_path.read_bytes())==meta['manifest_sha256']
    matrix.write_json(evidence/'job.json',meta)
    try:
        for variant in ['baseline','candidate']:
            binaries=build['builds'][variant]['binaries']
            for value in binaries.values():(stage/matrix.safe_member(value['file'])).chmod(0o755)
            meta['binary_paths'][variant]={key:str(stage/matrix.safe_member(value['file'])) for key,value in binaries.items()}
        meta['planned_commands']=worker_commands(bundle.index['assets'],meta['binary_paths'],selected,kind,work,evidence,models)
        check();matrix.write_json(evidence/'before-NN.json',meta);matrix.write_json(evidence/'job.json',meta)
        for planned in meta['planned_commands']:
            target=Path(planned['target']);target.parent.mkdir(parents=True,exist_ok=True)
            matrix.run_command(planned['argv'],planned['label'],evidence,meta,env)
            if kind.startswith('page'):
                matrix.read_page_results(target,kind[4:],selected)
                meta['successful_PageRecognizer_loads']+=1;meta['successful_Page_model_session_loads']+=2;meta['successful_Page_API_results']+=2*len(selected)
            else:
                matrix.read_cli_result(evidence/'commands'/(planned['label']+'.stdout'),target)
                meta['successful_CLI_processes']+=1
            matrix.write_json(evidence/'job.json',meta)
        check();meta.update(complete=True,frozen_materials_after_unchanged=True,image_results=meta['successful_Page_API_results']+meta['successful_CLI_processes'])
        assert meta['image_results']==len(selected)*(4 if kind.startswith('page') else 2)
    except Exception as error:meta['error']=repr(error);raise
    finally:matrix.write_json(evidence/'job.json',meta)
    return meta


def select_complete_jobs(records,bundle_sha,build_sha):
    selected=[];excluded=[];seen=set()
    for path,meta in records:
        reason='partial' if meta.get('complete') is not True else (
            'different_build_or_bundle' if meta.get('bundle_sha256')!=bundle_sha or meta.get('build_receipt_sha256')!=build_sha else None)
        if reason:
            excluded.append({'path':str(path),'reason':reason,'kind':meta.get('kind'),'shard':meta.get('shard'),
                             'bundle_sha256':meta.get('bundle_sha256'),'build_receipt_sha256':meta.get('build_receipt_sha256')})
            continue
        key=(meta.get('kind'),meta.get('shard',{}).get('index'))
        if key in seen:raise ValueError('duplicate complete job for current build')
        seen.add(key);selected.append((path,meta))
    return selected,excluded


def compare_all(bundle,stage,artifacts,output):
    import csv
    import json
    from pathlib import Path
    import struct
    import sys
    import vertical_page_matrix as matrix
    from verify_low_stop_evidence import metric_values
    stage=Path(stage).resolve();build=validate_stage(bundle,stage);build_sha=matrix.sha((stage/'build.json').read_bytes())
    artifacts=Path(artifacts);output=Path(output).resolve();output.mkdir(parents=True,exist_ok=False)
    records=[(path,json.loads(path.read_text())) for path in artifacts.rglob('job.json')]
    selected_records,excluded=select_complete_jobs(records,matrix.sha((bundle.folder/'bundle.json').read_bytes()),build_sha)
    matrix.write_json(output/'selection.json',{'selected':[str(p) for p,_ in selected_records],'excluded':excluded})
    metas=[meta for _,meta in selected_records];locations={(meta['kind'],meta['shard']['index']):path.parent for path,meta in selected_records}
    image_results=validate_complete_jobs(metas,bundle.plan,matrix.sha((bundle.folder/'bundle.json').read_bytes()),build_sha)
    predictions={kind:{} for kind in KINDS};page_records={};commands=[]
    def merge_unique(target,values):
        if set(target)&set(values):raise ValueError('duplicated physical prediction')
        target.update(values)
    for meta in metas:
        kind=meta['kind'];directory=locations[kind,meta['shard']['index']];selected=partition_inputs(bundle.index['physical_inputs'],meta['shard']['index'],meta['shard']['total'])
        assert meta['assets']==bundle.index['assets'];models=Path(meta['models_dir']);manifest=meta['manifest']
        assert meta['producer_sha256']==build['producer_sha256'] and meta['profile']=='release' and meta['platform'].startswith('Linux')
        assert meta['global_physical_inputs']==len(bundle.index['physical_inputs']) and meta['global_scoring_inputs']==len(bundle.index['scoring_inputs'])
        assert meta['physical_inputs']==len(selected)
        assert meta['scoring_inputs']==sum(r['physical_id'] in set(meta['shard']['physical_ids']) for r in bundle.index['scoring_inputs'])
        assert [r['id'] for r in manifest['inputs']]==[r['id'] for r in selected]
        assert all(set(r)=={'id','image'} for r in manifest['inputs'])
        assert matrix.sha((json.dumps(manifest,ensure_ascii=False,indent=2)+'\n').encode())==meta['manifest_sha256']
        work=Path(manifest['inputs'][0]['image']).parent.parent
        assert all(actual['image']==str(work/input_relative_path(row)) for actual,row in zip(manifest['inputs'],selected))
        assert meta['runtime_environment']=={'NDLOCR_INTRA_THREADS':'1','NDLOCR_PARSEQ_PARALLELISM':'1','LD_LIBRARY_PATH':str(Path(meta['stage_directory'])/'runtime')}
        assert meta['planned_commands']==worker_commands(bundle.index['assets'],meta['binary_paths'],selected,kind,work,work.parent/'evidence',models)
        before_nn=json.loads((directory/'before-NN.json').read_text());assert before_nn['planned_commands']==meta['planned_commands']
        for variant in ['baseline','candidate']:
            expected_binaries={key:str(Path(meta['stage_directory'])/value['file']) for key,value in build['builds'][variant]['binaries'].items()}
            assert meta['binary_paths'][variant]==expected_binaries
            if kind.startswith('page'):
                model=kind[4:];label=variant+'-'+kind;receipt=json.loads((directory/'commands'/(label+'.json')).read_text());declared_directory=work.parent/'evidence/page'/variant
                expected=[expected_binaries['page'],'--manifest',str(work/'NN-images-only.json'),'--det-model',str(models/bundle.index['assets']['deim']['name']),
                          '--parseq-model',str(models/bundle.index['assets'][model]['name']),'--charset',str(models/bundle.index['assets']['charset']['name']),
                          '--label','runtime'+model,'--output-dir',str(declared_directory)]
                matrix.verify_command(directory/'commands'/(label+'.json'),expected)
                values=matrix.read_page_results(directory/'page'/variant,model,selected)
                for setting in ['eval','default']:
                    key=variant+'_'+setting;rows=values[model+'_'+setting]
                    merge_unique(predictions[kind].setdefault(key,{}),{ident:r['text'] for ident,r in rows.items()})
                    merge_unique(page_records.setdefault((kind,setting,variant),{}),rows)
            else:
                values={}
                for row in selected:
                    ident=row['id'];label=variant+'-'+kind+'-'+ident;receipt=json.loads((directory/'commands'/(label+'.json')).read_text())
                    argv=receipt['argv'];body=work.parent/'evidence/cli'/variant/(ident+'.txt');expected=full_cli_command(expected_binaries['cli'],work/input_relative_path(row),models,bundle.index['assets'],kind,body)
                    matrix.verify_command(directory/'commands'/(label+'.json'),expected)
                    values[ident]=matrix.read_cli_result(directory/'commands'/(label+'.stdout'),directory/'cli'/variant/(ident+'.txt'))
                merge_unique(predictions[kind].setdefault(variant,{}),values)
        expected_commands=2 if kind.startswith('page') else 2*len(selected)
        assert len(meta['commands'])==expected_commands and all(r['exit_code']==0 for r in meta['commands'])
        receipts=[json.loads(p.read_text()) for p in (directory/'commands').glob('*.json')]
        assert len(receipts)==expected_commands
        assert {tuple(r['argv']) for r in receipts}=={tuple(r['argv']) for r in meta['commands']}
    def canonical(row):
        return {'text':row['text'],'page_index':row['page_index'],'lines':[{'bbox':r['bbox'],'text':r['text'],
                'confidence_bits':struct.unpack('!I',struct.pack('!f',r['confidence']))[0]} for r in row['lines']]}
    page_changes=[]
    for kind in KINDS[:3]:
        for setting in ['eval','default']:
            for row in bundle.index['physical_inputs']:
                ident=row['id'];before=canonical(page_records[kind,setting,'baseline'][ident]);after=canonical(page_records[kind,setting,'candidate'][ident])
                if before!=after:page_changes.append({'kind':kind,'setting':setting,'id':ident,'before':before,'after':after})
    truth=output/'metrics/truth';truth.mkdir(parents=True)
    for row in bundle.index['scoring_inputs']:(truth/(row['id']+'.txt')).write_bytes(bundle.read_truth(row))
    verified=0;pairs=[];regressions=[];gains=[]
    meta={'complete':False,'commands':commands,'bundle_sha256':matrix.sha((bundle.folder/'bundle.json').read_bytes()),'build_receipt_sha256':build_sha,'excluded_job_artifacts':excluded}
    matrix.write_json(output/'comparison.json',meta)
    try:
        for kind,systems in predictions.items():
            directories={}
            for key,values in systems.items():
                mapped=matrix.map_predictions(bundle.index['scoring_inputs'],values);directory=output/'predictions'/kind/key;directory.mkdir(parents=True);directories[key]=directory
                for ident,text in mapped.items():(directory/(ident+'.txt')).write_bytes(text.encode())
            for mode in ['none','width','strict']:
                table=output/'metrics'/kind/(mode+'.csv');table.parent.mkdir(parents=True,exist_ok=True)
                argv=[sys.executable,str(stage/'harness/tools/eval_ocr_metrics.py'),'--truth-dir',str(truth)]
                for key,directory in directories.items():argv+=['--system',key+'='+str(directory)]
                argv+=['--normalize',mode,'--require-all','--output-csv',str(table)];matrix.run_command(argv,kind+'-'+mode,output,meta)
                rows=list(csv.DictReader(table.open()));assert len(rows)==len(bundle.index['scoring_inputs'])*len(systems)
                assert len({(r['system'],r['file']) for r in rows})==len(rows)
                for r in rows:
                    expected=metric_values((truth/r['file']).read_text(),(directories[r['system']]/r['file']).read_text(),mode)
                    assert tuple(int(r[k]) for k in ['char_errors','char_total','word_errors','word_total'])==expected;verified+=1
                indexed={(r['system'],r['file']):r for r in rows}
                for setting in (['eval','default'] if kind.startswith('page') else [None]):
                    before_key='baseline'+('_'+setting if setting else '');after_key='candidate'+('_'+setting if setting else '')
                    for score in bundle.index['scoring_inputs']:
                        name=score['id']+'.txt';before=indexed[before_key,name];after=indexed[after_key,name]
                        entry={'kind':kind,'setting':setting,'normalization':mode,'scoring_id':score['id'],'physical_id':score['physical_id'],'origin':score['origin'],
                               'before':before,'after':after,'body_unchanged':systems[before_key][score['physical_id']]==systems[after_key][score['physical_id']]};pairs.append(entry)
                        if any(int(after[k])>int(before[k]) for k in ['char_errors','word_errors']):regressions.append(entry)
                        if mode=='none' and int(after['char_errors'])<int(before['char_errors']) and int(after['word_errors'])<=int(before['word_errors']):gains.append(entry)
        require_complete_metric_count(verified,bundle.plan)
        meta.update(complete=True,image_results=image_results,verified_CSV_rows=verified,scoring_comparison_pairs=len(pairs),regression_pairs=len(regressions),
                    page_body_bbox_confidence_changes=len(page_changes),raw_gain_scoring_pairs=len(gains),gains_are_known_development_inputs=True,
                    full_gate_pass=not regressions and not page_changes,new72_gate_pass=True,
                    successful_Page_API_results=sum(m['successful_Page_API_results'] for m in metas),successful_CLI_processes=sum(m['successful_CLI_processes'] for m in metas),
                    successful_PageRecognizer_loads=sum(m['successful_PageRecognizer_loads'] for m in metas),successful_Page_model_session_loads=sum(m['successful_Page_model_session_loads'] for m in metas),
                    internal_forward_calls=None,CLI_model_session_loads=None,producer_sha256=matrix.sha(Path(__file__).read_bytes()))
        for name,values in [('all-input-pairs',pairs),('regressions',regressions),('raw-known-gains',gains),('page-changes',page_changes)]:matrix.write_json(output/(name+'.json'),values)
        matrix.write_json(output/'summary.json',{k:v for k,v in meta.items() if k!='commands'})
    except Exception as error:meta['error']=repr(error);raise
    finally:matrix.write_json(output/'comparison.json',meta)
    return meta


if __name__=='__main__':
    import argparse
    import json
    from pathlib import Path
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--bundle',type=Path,default=Path(__file__).resolve().parents[1]/'docs/benchmarks/vertical-cli-fallback/full-matrix-v1')
    parser.add_argument('--validate',action='store_true');parser.add_argument('--build',action='store_true')
    parser.add_argument('--kind',choices=KINDS);parser.add_argument('--index',type=int);parser.add_argument('--total',type=int,default=8)
    parser.add_argument('--stage',type=Path);parser.add_argument('--artifacts',type=Path);parser.add_argument('--output',type=Path);parser.add_argument('--models',type=Path,default=Path('models'))
    args=parser.parse_args()
    try:
        bundle=FullBundle(args.bundle)
        if args.validate:print(json.dumps({'validated':True,'expected':bundle.index['expected'],'jobs':len(bundle.plan['jobs']),'NN_calls':0}))
        elif args.build and args.output:build_pair(bundle,args.output)
        elif args.kind and args.output and args.stage and args.index is not None:run_shard(bundle,args.stage,args.kind,args.index,args.total,args.output,args.models)
        elif args.artifacts and args.output and args.stage:compare_all(bundle,args.stage,args.artifacts,args.output)
        else:parser.error('choose --validate, --build, --kind or --artifacts with required paths')
    except Exception as error:
        if args.output:
            from vertical_page_matrix import write_json
            args.output.mkdir(parents=True,exist_ok=True)
            write_json(args.output/'failure.json',{'complete':False,'error':repr(error),'kind':args.kind,'index':args.index,'total':args.total})
        raise
