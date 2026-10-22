"""Verify literal traces and original pixels, replay actual Rust policy without NN."""
from pathlib import Path, PurePosixPath
from collections import Counter
import argparse, csv, gzip, hashlib, io, json, math, os, struct, subprocess, tempfile, zipfile
from PIL import Image
f32=lambda x:struct.unpack('<f',struct.pack('<f',x))[0]
bits=lambda x:struct.unpack('<I',struct.pack('<f',x))[0]
floatbits=lambda x:struct.unpack('<f',struct.pack('<I',x))[0]
sha=lambda b:hashlib.sha256(b).hexdigest()
def audit(record, limit):
    assert record['api_calls'] == 1 and record['internal_session_run_calls'] is None
    if 'error' in record:
        return
    raw, cursor, total, emitted = record['text'].encode(), 0, 0.0, 0
    steps = record['steps']
    assert 0 < len(steps) <= limit
    for t, step in enumerate(steps):
        assert step['timestep'] == t and len(step['candidates']) == 1
        candidate = step['candidates'][0]
        probability = floatbits(candidate['probability_bits'])
        assert bits(candidate['probability']) == candidate['probability_bits'] and 0 <= probability <= 1
        span = step['text_range']
        if span is not None:
            assert span[0] == cursor and len(candidate['character']) == 1 and candidate['token_id'] > 0
            assert raw[span[0]:span[1]].decode() == candidate['character']
            assert emitted < 2 or probability >= f32(0.35)
            cursor = span[1]; emitted += 1; total = f32(total + probability)
        else:
            assert t == len(steps) - 1
    assert cursor == len(raw) and emitted == len(record['text'])
    assert bits(f32(total / emitted) if emitted else 0.0) == record['mean_bits']
    last = steps[-1]; winner = last['candidates'][0]
    if record['stop_reason'] == 'EndOfSequence':
        assert last['text_range'] is None and winner['token_id'] == 0 and winner['character'] is None
    elif record['stop_reason'] == 'LowConfidence':
        assert last['text_range'] is None and winner['token_id'] > 0 and floatbits(winner['probability_bits']) < f32(0.35) and emitted >= 2
    else:
        assert record['stop_reason'] == 'TimestepsExhausted' and len(steps) == emitted == limit

def verify_native(packet,output,truth,expected,scorer):
    stage=json.loads((packet/'native-stage.json').read_text());assert sha(scorer.read_bytes())==stage['scorer_sha256']
    for name,h in stage['files_sha256'].items():assert sha((packet/name).read_bytes())==h,name
    data=json.loads(gzip.decompress((packet/'native-records.json.gz').read_bytes()));mapping={v:{(x['id'],x['label']):x for x in xs}for v,xs in data.items()};expected={(x['id'],x['model']):x for x in expected}
    assert all(len(x)==1242 and set(x)==set(expected)for x in mapping.values())
    routes=('raw_auto','raw_horizontal','normal_auto','normal_horizontal','fragment_auto','fragment_horizontal','fragment_batch_auto','fragment_exclusive_batch_auto');allowed={'fragment_auto','fragment_batch_auto','fragment_exclusive_batch_auto'};changed=[]
    for key,choice in expected.items():
        a,b=mapping['baseline'][key],mapping['candidate'][key]
        for v in (a,b):
            assert v['raw_auto']==choice['base']and v['fragment_auto']==v['fragment_batch_auto']==v['fragment_exclusive_batch_auto']
            for route in routes:
                x=v[route];assert set(x)=={'text','mean_bits'};mean=floatbits(x['mean_bits']);assert math.isfinite(mean)and 0<=mean<=1
        for route in routes:
            wanted=choice['result']if route in allowed and choice['adopted']else a[route];assert b[route]==wanted
            if b[route]!=a[route]:changed.append((key,route))
    assert len(changed)==27 and len({k for k,route in changed})==9
    metricdir=output/'native-metrics';metricdir.mkdir();truthdir=metricdir/'truth';truthdir.mkdir()
    for ident,data in truth.items():(truthdir/(ident+'.txt')).write_bytes(data)
    for variant,rows in mapping.items():
        for (ident,label),row in rows.items():
            directory=metricdir/(variant+'-'+label);directory.mkdir(exist_ok=True);(directory/(ident+'.txt')).write_bytes(row['fragment_auto']['text'].encode())
    from fast_distance import distance
    import string
    def normalize_independently(text,level):
        if level=='none':return text
        if level=='width':return ''.join(chr(ord(c)-65248)if 65281<=ord(c)<=65374 else c for c in text)
        folded='\n'.join(' '.join(line.split())for line in text.replace('\r\n','\n').replace('\r','\n').split('\n'))
        return ''.join(c for c in folded if c not in string.punctuation and c!=' ').strip()
    independent_rows=0
    comparisons=0;regressions=[];metric_columns=('char_errors','char_total','word_errors','word_total')
    for label in ('30','50','100'):
        for norm in ('none','width','strict'):
            table=metricdir/f'model{label}-{norm}.csv';argv=[os.sys.executable,str(scorer),'--truth-dir',str(truthdir),'--system','baseline='+str(metricdir/f'baseline-{label}'),'--system','candidate='+str(metricdir/f'candidate-{label}'),'--normalize',norm,'--require-all','--baseline-system','baseline','--output-csv',str(table),'--output-md',str(table.with_suffix('.md'))]
            done=subprocess.run(argv,capture_output=True);(metricdir/f'model{label}-{norm}.stdout').write_bytes(done.stdout);(metricdir/f'model{label}-{norm}.stderr').write_bytes(done.stderr);assert done.returncode==0
            rows=csv_rows(table);fixed=csv_rows(packet/'native-metrics'/table.name);assert len(rows)==len(fixed)==828
            lookup={(x['system'],x['file']):x for x in fixed}
            for x in rows:
                assert all(x[k]==lookup[x['system'],x['file']][k]for k in metric_columns)
                a=normalize_independently((truthdir/x['file']).read_text(),norm)
                b=normalize_independently((metricdir/(x['system']+'-'+label)/x['file']).read_text(),norm)
                values=dict(char_errors=distance(a,b),char_total=max(1,len(a)),word_errors=distance(a.split(),b.split()),word_total=max(1,len(a.split())))
                assert all(int(x[k])==v for k,v in values.items()),(label,norm,x,values)
                independent_rows+=1
            pairs={(x['system'],x['file']):x for x in rows}
            for ident in truth:
                a,b=[pairs[v,ident+'.txt']for v in ('baseline','candidate')];comparisons+=2
                if int(b['char_errors'])>int(a['char_errors'])or int(b['word_errors'])>int(a['word_errors']):regressions.append(dict(id=ident,model=label,normalization=norm))
    assert comparisons==7452 and independent_rows==7452 and not regressions
    return dict(native_independently_verified_CSV_rows=independent_rows,native_independent_error_and_denominator_fields=independent_rows*4,native_route_fields_compared=1242*8,native_changed_pairs=9,native_five_untouched_routes=6210,native_official_char_and_word_comparisons=7452,native_per_input_regressions=0)



def verify_page(packet,output,scorer):
    from contextlib import ExitStack
    from fast_distance import distance
    import string
    stage=json.loads((packet/'page-stage.json').read_text());assert sha(scorer.read_bytes())==stage['scorer_sha256']
    assert stage['fresh_candidate_artifacts_verified']
    binding=json.loads((packet/'page-artifact-binding.json').read_text())
    assert binding['all_primary_artifacts_fresh_false'] and binding['both_binaries_differ_from_baseline']
    for name in ('ndlocr-lite-rs','capture_page_matrix'):
        assert binding['candidate'][name]['fresh'] is False
        assert binding['baseline'][name]['sha256']!=binding['candidate'][name]['sha256']
    source_index=json.loads((packet/'source-overlay.json').read_text())['members']
    for name,h in binding['candidate_Rust_source_sha256'].items():assert source_index[name]['sha256']==h,name
    summary=json.loads((packet/'page-summary.json').read_text());assert summary['page_gate_pass']

    for name,h in stage['files_sha256'].items():assert sha((packet/name).read_bytes())==h,name
    index=json.loads((packet/'page-inputs.json').read_text());inputs=index['inputs'];assert len(inputs)==444
    truth={};dimensions={};pixel_ids=set()
    with ExitStack()as stack:
        archives={}
        for x in inputs:
            ident=x['id'];assert ident and all(c.isascii()and(c.isalnum()or c in '_-')for c in ident)
            payloads={}
            for kind in ('image','truth'):
                ref=x['payloads'][kind];name=ref['archive'];assert not Path(name).is_absolute()and '..'not in Path(name).parts
                if name not in archives:archives[name]=stack.enter_context(zipfile.ZipFile(packet/name))
                data=archives[name].read(ref['member']);assert sha(data)==ref['sha256']==x[kind+'_sha256']and len(data)==ref['bytes'];payloads[kind]=data
            with Image.open(io.BytesIO(payloads['image']))as im:
                rgb=im.convert('RGB');assert list(rgb.size)==x['identity']['dimensions'];h=sha(rgb.tobytes());assert h==x['identity']['RGB_sha256'];pixel_ids.add(h);dimensions[ident]=rgb.size
            truth[ident]=payloads['truth']
    assert len(pixel_ids)==443
    # Verify the old293 baseline source identity using captured bytes, not mutable main paths.
    history=json.loads(gzip.decompress((packet/'history-index.json.gz').read_bytes()));proof=json.loads((packet/'page-baseline-reuse-proof.json').read_text())
    with ExitStack()as stack:
        archives={};cache={}
        def captured(h):
            if h not in cache:
                ref=history['backing_objects'][h];name=ref['archive']
                if name not in archives:archives[name]=stack.enter_context(zipfile.ZipFile(packet/name))
                data=archives[name].read(ref['member']);assert sha(data)==h;cache[h]=data
            return cache[h]
        old={name:h for name,h in proof['old_source_sha256'].items()if name.startswith('src/')and name.endswith('.rs')};e5=proof['current_main_source_sha256']
        transforms={
            'src/infer/cached.rs':lambda t:t+'\n#[cfg(test)]\n#[path = "../../tests/support/output_capacity_metadata.rs"]\nmod output_capacity_metadata;\n',
            'src/infer/vertical_capacity.rs':lambda t:t.replace('mod policy;','mod policy;\n#[cfg(test)]\nmod prefix;',1),
            'src/infer/mod.rs':lambda t:t+'\n#[cfg(test)]\nmod output_capacity;\n'}
        for name,h in old.items():
            a,b=captured(h),captured(e5[name])
            if name in transforms:assert transforms[name](a.decode())==b.decode(),name
            else:assert a==b,name
        assert set(e5)-set(old)=={'src/infer/output_capacity.rs','src/infer/vertical_capacity/prefix.rs'}
        overlay=json.loads((packet/'source-overlay.json').read_text())['members']
        for name in ('Cargo.toml','Cargo.lock'):assert overlay[name]['sha256']==proof['old_source_sha256'][name]
    records=json.loads(gzip.decompress((packet/'page-records.json.gz').read_bytes()));routes=('30_eval','30_default','50_eval','50_default','100_eval','100_default','cli_deim','cli_bands');mapping={};changes=0
    for variant in ('baseline','candidate'):
        mapping[variant]={}
        for route in routes:
            rows=records[variant][route];byid={x['id']:x for x in rows};assert len(rows)==len(byid)==444 and set(byid)==set(truth);mapping[variant][route]=byid
            for ident,row in byid.items():
                if route.startswith('cli_'):
                    body=row['text'].encode();assert row['transport_LF_bytes']==1 and sha(body)==row['body_sha256']and sha(body+b'\n')==row['stdout_sha256']
                else:
                    assert row['text']=='\n'.join(line['text']for line in row['lines']);w,h=dimensions[ident]
                    for line in row['lines']:
                        assert math.isfinite(line['confidence'])and 0<=line['confidence']<=1
                        x0,y0,x1,y1=line['bbox'];assert 0<=x0<x1<=w and 0<=y0<y1<=h
    text_changes=[];metadata=[]
    for route in routes:
        for ident in truth:
            a,b=[mapping[v][route][ident]for v in ('baseline','candidate')]
            if a['text']!=b['text']:text_changes.append(dict(id=ident,route=route,before=a['text'],after=b['text']))
            if not route.startswith('cli_'):
                model,mode=route.split('_')
                metadata.append(dict(id=ident,model=model,mode=mode,page_index_equal=a['page_index']==b['page_index'],line_count_equal=len(a['lines'])==len(b['lines']),bbox_equal=[x['bbox']for x in a['lines']]==[x['bbox']for x in b['lines']],confidence_bits_equal=[bits(x['confidence'])for x in a['lines']]==[bits(x['confidence'])for x in b['lines']],line_texts_equal=[x['text']for x in a['lines']]==[x['text']for x in b['lines']]))
    ordered=lambda rows:sorted(rows,key=lambda x:json.dumps(x,sort_keys=True))
    assert ordered(text_changes)==ordered(summary['text_changes'])
    assert ordered(metadata)==ordered(summary['metadata'])
    changes=len(text_changes)

    directory=output/'page-metrics';directory.mkdir();gt=directory/'truth';gt.mkdir()
    for ident,data in truth.items():(gt/(ident+'.txt')).write_bytes(data)
    for variant,systems in mapping.items():
        for route,rows in systems.items():
            folder=directory/(variant+'_'+route);folder.mkdir()
            for ident,row in rows.items():(folder/(ident+'.txt')).write_bytes(row['text'].encode())
    def normal(text,level):
        if level=='none':return text
        if level=='width':return ''.join(chr(ord(c)-65248)if 65281<=ord(c)<=65374 else c for c in text)
        folded='\n'.join(' '.join(line.split())for line in text.replace('\r\n','\n').replace('\r','\n').split('\n'))
        return ''.join(c for c in folded if c not in string.punctuation and c!=' ').strip()
    count=0;regressions=[];improvements=[];cohorts={x['id']:x['cohort']for x in inputs}
    for norm in ('none','width','strict'):
        table=directory/(norm+'.csv');argv=[os.sys.executable,str(scorer),'--truth-dir',str(gt)]
        for variant in ('baseline','candidate'):
            for route in routes:argv+=['--system',variant+'_'+route+'='+str(directory/(variant+'_'+route))]
        argv+=['--normalize',norm,'--require-all','--baseline-system','baseline_cli_deim','--output-csv',str(table),'--output-md',str(table.with_suffix('.md'))]
        p=subprocess.run(argv,capture_output=True);(directory/(norm+'.stdout')).write_bytes(p.stdout);(directory/(norm+'.stderr')).write_bytes(p.stderr);assert p.returncode==0
        actual=csv_rows(table);fixed={(x['system'],x['file']):x for x in csv_rows(packet/'page-metrics'/table.name)};assert len(actual)==len(fixed)==444*16
        for x in actual:
            a=normal((gt/x['file']).read_text(),norm);b=normal((directory/x['system']/x['file']).read_text(),norm);values=dict(char_errors=distance(a,b),char_total=max(1,len(a)),word_errors=distance(a.split(),b.split()),word_total=max(1,len(a.split())))
            assert all(int(x[k])==v and x[k]==fixed[x['system'],x['file']][k]for k,v in values.items()),(norm,x,values);count+=1
        metric_index={(x['system'],x['file']):x for x in actual}
        for route in routes:
            for ident in truth:
                before,after=[metric_index[v+'_'+route,ident+'.txt']for v in ('baseline','candidate')]
                assert before['char_total']==after['char_total']and before['word_total']==after['word_total']
                pair=dict(id=ident,cohort=cohorts[ident],route=route,normalization=norm,baseline_char_errors=int(before['char_errors']),candidate_char_errors=int(after['char_errors']),char_total=int(before['char_total']),baseline_word_errors=int(before['word_errors']),candidate_word_errors=int(after['word_errors']),word_total=int(before['word_total']))
                if pair['candidate_char_errors']>pair['baseline_char_errors']or pair['candidate_word_errors']>pair['baseline_word_errors']:regressions.append(pair)
                if pair['candidate_char_errors']<pair['baseline_char_errors']or pair['candidate_word_errors']<pair['baseline_word_errors']:improvements.append(pair)
    assert count==21312
    assert ordered(regressions)==ordered(summary['per_input_regressions'])and ordered(improvements)==ordered(summary['per_input_improvements'])
    assert not regressions
    return dict(Page_input_IDs=444,Page_unique_RGB=443,Page_recorded_image_results=444*8*2,Page_text_changes=changes,Page_per_input_regressions=0,Page_independently_verified_CSV_rows=count,Page_old293_baseline_source_replayed=True,Page_fresh_candidate_artifacts_verified=True)



def csv_rows(path):
    with path.open(newline='',encoding='utf-8')as stream:
        return list(csv.DictReader(stream))

def select_prebuilt_binary(packet, supplied):
    binary=supplied.resolve()
    binding=json.loads((packet/'prebuilt-replay-binary.json').read_text())
    allowed={binding['sha256']}|{x['sha256']for x in binding.get('also_allowed',[])}
    assert sha(binary.read_bytes())in allowed,'unrecognized lib test binary'
    return binary


def select_source(packet, output, explicit_manifest):
    if explicit_manifest is not None:
        manifest=explicit_manifest.resolve()
        assert manifest.is_file(),manifest
        return manifest
    index=json.loads((packet/'source-overlay.json').read_text())['members']
    destination=output/'replay-source'
    assert not destination.exists()
    validated={}
    with zipfile.ZipFile(packet/'source-overlay.zip')as archive:
        assert len(archive.namelist())==len(index)and set(archive.namelist())==set(index)
        for info in archive.infolist():
            name=info.filename;path=PurePosixPath(name)
            assert name and not path.is_absolute()and '..'not in path.parts and '\\'not in name
            assert path.as_posix()==name and not info.is_dir()
            assert not(len(path.parts[0])==2 and path.parts[0][1]==':')
            assert (info.external_attr>>16)&0o170000!=0o120000
            data=archive.read(info);record=index[name]
            assert len(data)==record['bytes']and sha(data)==record['sha256'],name
            validated[name]=data
    assert 'Cargo.toml'in validated
    destination.mkdir()
    for name,data in validated.items():
        target=destination/name;target.parent.mkdir(parents=True,exist_ok=True);target.write_bytes(data)
    return destination/'Cargo.toml'


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--manifest-path',type=Path,help='optional matching checkout; default restores the frozen source under output')
    parser.add_argument('--rust-test-binary',type=Path,help='optional already-built matching lib test binary')
    parser.add_argument('--scorer-script',type=Path,help='optional matching scorer; default uses the selected source')
    parser.add_argument('--output-dir',type=Path,required=True)
    args=parser.parse_args();output=args.output_dir.resolve();output.mkdir()
    packet=Path(__file__).resolve().parent;manifest_path=select_source(packet,output,args.manifest_path);source=manifest_path.parent
    scorer=args.scorer_script.resolve()if args.scorer_script else source/'tools/eval_ocr_metrics.py'
    for rel,h in json.loads((packet/'policy-source.json').read_text()).items():assert sha((source/rel).read_bytes())==h,rel
    frozen=json.loads((packet/'data-stage.json').read_text())
    for name,h in frozen['archive_sha256'].items():assert sha((packet/name).read_bytes())==h,name
    bindings=json.loads((packet/'crop-bindings.json').read_text());assert len(bindings)==3190
    assert sha((packet/'crop-bindings.json').read_bytes())==json.loads((packet/'crop-bindings-summary.json').read_text())['sha256']
    inp=json.loads((packet/'inputs.json').read_text());inputs={x['id']:x for x in inp['inputs']};assert len(inputs)==414
    actual=json.loads((packet/'horizontal-inputs.json').read_text())['members'];assert len(actual)==1653
    literal=next(l.split(':',1)[1].strip()for l in (packet/'charset.yaml').read_text().splitlines()if l.strip().startswith('charset_test:'))
    charset=json.loads(literal);assert len(charset)==7141
    metadata={x['label']:x for x in json.loads((packet/'metadata.json').read_text())}
    rotated={};truth={};rgb={}
    with zipfile.ZipFile(packet/'inputs.zip')as z:
        assert set(z.namelist())==set(inp['members'])
        for name,record in inp['members'].items():
            path=Path(name);assert not path.is_absolute()and '..'not in path.parts
            data=z.read(name);assert sha(data)==record['sha256']and len(data)==record['bytes']
        for ident,x in inputs.items():
            with Image.open(io.BytesIO(z.read(x['image'])))as im:
                im=im.convert('RGB');assert im.size==(x['width'],x['height'])and im.tobytes()==z.read(x['rgb'])
                rotated[ident]=im.transpose(Image.Transpose.ROTATE_90)
            truth[ident]=z.read(x['truth']);rgb[ident]=z.read(x['rgb'])
    with zipfile.ZipFile(packet/'horizontal-inputs.zip')as z:
        assert set(z.namelist())==set(actual)
        for name,x in actual.items():
            data=z.read(name);assert sha(data)==x['sha256']and len(data)==x['bytes']==x['width']*x['height']*3
        for b in bindings:
            image=rotated[b['source_id']];a,c=b['x_range'];assert image.crop((a,0,c,image.height)).tobytes()==z.read(b['actual_input_member'])
    rows={};stats=Counter()
    with gzip.open(packet/'raw-traces.jsonl.gz','rt')as f:
        for line in f:
            item=json.loads(line);record=item['record'];label=item['model'];orientation=item['orientation'];key=label,orientation,record['id'];assert key not in rows
            limit=metadata[label]['outputs'][0]['max_raw_scalars_from_static_shape'];audit(record,limit)
            for step in record['steps']:
                c=step['candidates'][0];token=c['token_id']
                assert c['character']==(None if token==0 else charset[token-1])
            if orientation=='Auto':assert record['id']in inputs and item['actual_input_member']==inputs[record['id']]['rgb']
            else:
                x=actual[item['actual_input_member']];assert orientation=='Horizontal'and record['orientation']=='Horizontal'
                assert record['expected_rgb_sha256']==x['sha256']and(record['width'],record['height'])==(x['width'],x['height'])
            rows[key]=record;stats[orientation]+=1
    assert stats=={'Auto':1242,'Horizontal':3190}
    combined=json.loads(gzip.decompress((packet/'combine-manifest.json.gz').read_bytes()));expected=json.loads(gzip.decompress((packet/'expected-decisions.json.gz').read_bytes()))
    links=json.loads(gzip.decompress((packet/'route-links.json.gz').read_bytes()));assert len(combined['items'])==len(expected)==len(links)==1242
    recognition=lambda r:{k:v for k,v in r.items()if k in ('text','mean_bits','error')}
    for item,link in zip(combined['items'],links):
        assert(item['id'],item['model'],item['plan'])==(link['id'],link['model'],link['plan'])
        assert item['base']==recognition(rows[item['model'],'Auto',item['id']])
        if item['plan']:
            assert len(item['routes'])==len(link['routes'])==2
            for route,parts in zip(link['routes'],item['routes']):
                assert len(route)==len(parts)
                for c,part in zip(route,parts):
                    raw=rows[item['model'],'Horizontal',c['record_id']];assert recognition(raw)==part
                    image=rotated[item['id']];a,b=c['x_range'];data=image.crop((a,0,b,image.height)).tobytes()
                    assert sha(data)==raw['expected_rgb_sha256']and(c['width'],c['height'])==(raw['width'],raw['height'])==(b-a,image.height)
    # The Rust recorder requires raw RGB files, not GT. Bound paths stay in this temporary directory.
    with tempfile.TemporaryDirectory(prefix='vertical-prefix-replay-')as temporary:
        temp=Path(temporary);rgbdir=temp/'rgb';rgbdir.mkdir()
        for ident,data in rgb.items():(rgbdir/(ident+'.rgb')).write_bytes(data)
        real=[]
        for item in combined['items']:
            x=inputs[item['id']];dims=metadata[item['model']]['inputs'][0]['dimensions']
            real.append(dict(id=item['id'],model=item['model'],rgb=str(rgbdir/(item['id']+'.rgb')),width=x['width'],height=x['height'],input=[dims[3],dims[2]],limit=item['limit'],base=item['base']))
        combine_file=temp/'combine.json';combine_file.write_text(json.dumps(combined,ensure_ascii=False))
        plan_file=temp/'plan.json';plan_file.write_text(json.dumps(dict(expected_items=1242,items=real),ensure_ascii=False))
        env=os.environ|{'CARGO_PROFILE_DEV_DEBUG':'0','CARGO_PROFILE_TEST_DEBUG':'0'}
        if args.rust_test_binary:
            binary=select_prebuilt_binary(packet,args.rust_test_binary)
        else:
            argv=['cargo','test','--locked','--manifest-path',str(manifest_path),'--lib','--no-run','--message-format=json']
            p=subprocess.run(argv,capture_output=True,env=env);(output/'build.stdout').write_bytes(p.stdout);(output/'build.stderr').write_bytes(p.stderr);assert p.returncode==0
            events=[json.loads(l)for l in p.stdout.decode().splitlines()if l.startswith('{')]
            artifacts=[e for e in events if e.get('reason')=='compiler-artifact'and e.get('executable')and'lib'in e['target']['kind']];assert len(artifacts)==1;binary=Path(artifacts[0]['executable'])
        commands=[]
        for kind,name,variables in [('combine','combine_all_saved_real_plans_without_nn',{'NDLOCR_VERIFIED_COMBINE_MANIFEST':str(combine_file),'NDLOCR_VERIFIED_COMBINE_OUTPUT':str(output/'combined')}),('plans','record_real_raw_plans_without_nn',{'NDLOCR_VERIFIED_REAL_MANIFEST':str(plan_file),'NDLOCR_VERIFIED_REAL_OUTPUT':str(output/'real-plans')})]:
            argv=[str(binary),'infer::vertical_capacity::prefix::capture::'+name,'--exact','--ignored','--nocapture'];p=subprocess.run(argv,env=env|variables,capture_output=True);(output/(kind+'.stdout')).write_bytes(p.stdout);(output/(kind+'.stderr')).write_bytes(p.stderr);commands.append(dict(argv=argv,exit_code=p.returncode));assert p.returncode==0
        assert json.loads((output/'combined/records.json').read_text())==expected
        plans=json.loads((output/'real-plans/records.json').read_text());assert len(plans)==1242
        with zipfile.ZipFile(output/'real-plans/chunks.zip')as z:
            for plan,link in zip(plans,links):
                assert(plan['id'],plan['model'],plan['plan'])==(link['id'],link['model'],link['plan'])
                if plan['plan']:
                    for route,stored in zip(plan['routes'],link['routes']):
                        assert len(route)==len(stored)
                        for c,b in zip(route,stored):
                            assert c['x_range']==b['x_range'];image=rotated[plan['id']];a,end=c['x_range'];assert z.read(c['file'])==image.crop((a,0,end,image.height)).tobytes()
    native=verify_native(packet,output,truth,expected,scorer)
    page=verify_page(packet,output,scorer)
    summary=dict(**page,**native,source_mode='matching explicit checkout'if args.manifest_path else'frozen source restored under output',Rust_binary_sha256=sha(binary.read_bytes()),original_RGB=414,literal_traces_verified=4432,all_actual_horizontal_crop_bindings=3190,Rust_decisions_verified=1242,Rust_plans_verified=1242,adoptions=sum(x['adopted']for x in expected),new_NN_calls=0,new_model_loads=0,internal_session_run_calls=0,commands=commands)
    (output/'verification.json').write_text(json.dumps(summary,indent=2)+'\n');print(json.dumps(summary,indent=2))

if __name__=='__main__':main()
