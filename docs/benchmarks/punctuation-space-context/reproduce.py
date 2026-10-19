#!/usr/bin/env python3
"""Verify archived witnesses and replay official character/word scores without models."""
import argparse,hashlib,json,math,os,shutil,struct,subprocess,sys,zipfile
from pathlib import Path

def digest(data):return hashlib.sha256(data).hexdigest()
def read(path):return json.loads(path.read_text(encoding='utf-8'))
def safe(key):
    p=Path(key);assert not p.is_absolute()and '..'not in p.parts;return p

def extract(here,name,target):
    idx=read(here/(name+'.json'));archive=here/(name+'.zip');assert digest(archive.read_bytes())==idx['sha256']
    target.mkdir()
    with zipfile.ZipFile(archive)as z:
        assert len(z.namelist())==len(set(z.namelist()))==idx['member_count'];assert set(z.namelist())==set(idx['members'])
        for key,h in idx['members'].items():
            b=z.read(key);assert digest(b)==h,key;p=target/safe(key);p.parent.mkdir(parents=True,exist_ok=True);p.write_bytes(b)
    if(target/'aliases.json').exists():
        for key,source in read(target/'aliases.json').items():
            p=target/safe(key);b=(target/safe(source['source'])).read_bytes();assert digest(b)==source['sha256'];p.parent.mkdir(parents=True,exist_ok=True);assert not p.exists();p.write_bytes(b)
    return idx

def geometry(rgb,w,h,component,details=False):
    assert len(rgb)==w*h*3
    mask=bytearray(sum(rgb[i:i+3])<660 for i in range(0,len(rgb),3))
    if not component:
        ys=[i//w for i,v in enumerate(mask)if v];assert ys;body=max(ys)-min(ys)+1
        columns=[any(mask[y*w+x]for y in range(h))for x in range(w)];n=None
    else:
        comps=[]
        for seed in range(len(mask)):
            if not mask[seed]:continue
            mask[seed]=0;pending=[seed];x0,y0,x1,y1,area=w,h,0,0,0
            while pending:
                i=pending.pop();x,y=i%w,i//w;x0=min(x0,x);y0=min(y0,y);x1=max(x1,x+1);y1=max(y1,y+1);area+=1
                for yy in range(max(0,y-1),min(h,y+2)):
                    for xx in range(max(0,x-1),min(w,x+2)):
                        j=yy*w+xx
                        if mask[j]:mask[j]=0;pending.append(j)
            if area>=3:comps.append((x0,y0,x1,y1,area))
        body=max(c[3]-c[1]for c in comps);comps=[c for c in comps if c[4]>=max(3,(body+1)//2)];n=len(comps);columns=[False]*w
        for c in comps:columns[c[0]:c[2]]=[True]*(c[2]-c[0])
    xs=[i for i,v in enumerate(columns)if v];gap=current=0
    for x in range(xs[0],xs[-1]+1):
        current=0 if columns[x]else current+1;gap=max(gap,current)
    if details:
        widths=sorted((c[2]-c[0]for c in comps if 4*(c[3]-c[1])>=3*body and c[2]-c[0]<=body),reverse=True)
        return body,gap,n,xs[-1]-xs[0]+1,(widths[1]if len(widths)>1 else None),len(widths)
    return body,gap,n


def verify_variants(stage):
    m=read(stage/'new/data/evaluation-manifest.json');cases={x['id']:x for x in m['inputs']}
    for v in m['variants']:
        x=cases[v['base_id']];rgb=(stage/'corpus/rgb'/(x['id']+'.rgb')).read_bytes();w,h=x['width'],x['height'];xp=0 if v['variant']=='base'else 2;yp={'base':0,'x2y0':0,'x2y1':1,'x2y2':2}[v['variant']];pw=w+2*xp;ph=h+2*yp;edge=b'\xff'*(xp*3);white=b'\xff'*(pw*3)
        body=b''.join(edge+rgb[y*w*3:(y+1)*w*3]+edge for y in range(h));padded=white*yp+body+white*yp
        assert list(v['dimensions'])==[pw,ph]and digest(padded)==v['rgb_sha256']

def replay_raw(root):
    for label,count,new,rows in [('C',782,480,2346),('D',1358,576,4074)]:
        stage=root/label;m=read(stage/'evaluation-manifest.json');assert len(m['inputs'])==count
        gm={x['id']:x for x in read(stage/'geometry.json')}
        for x in m['inputs']:
            for kind,folder,suffix in [('image','png','.png'),('rgb','rgb','.rgb'),('truth','truth','.txt')]:
                p=stage/'corpus'/folder/(x['id']+suffix);assert digest(p.read_bytes())==x[kind+'_sha256']
            body,gap,n=geometry((stage/'corpus/rgb'/(x['id']+'.rgb')).read_bytes(),x['width'],x['height'],label=='D');g=gm[x['id']]
            assert body==g['foreground_height'if label=='C'else'body_height']and gap==g['max_blank_gap']
            assert g['supported']==(0<4*gap<=3*body)
            if label=='D':assert n==g['components']
        source=read(stage/'geometry-command.json')
        for p,h in source['sha256'].items():
            prefix='tmp/'+('ink-height-space'if label=='C'else'punctuation-space-components')+'/'
            target=stage/p[len(prefix):]if p.startswith(prefix)else stage/'geometry-source'/p
            assert digest(target.read_bytes())==h,(label,p)
        capture=read(stage/'raw-command.json');assert digest((stage/'run_raw.py').read_bytes())==capture['driver_sha256']
        raw_source=capture['raw_source_capture']
        for p,h in raw_source['source_sha256'].items():assert digest((root/'parent/source/after'/p).read_bytes())==h
        nm=read(stage/'new/data/evaluation-manifest.json');assert len(nm['inputs'])==new
        assert digest((stage/'new/data/atlas.png').read_bytes())==nm['atlas_sha256'];assert digest((stage/'new/data/lines.json').read_bytes())==nm['lines_sha256']
        for model in ['30','50','100']:
            assert read(stage/'new'/(model+'-command.json'))['exit_code']==0
            exported=read(stage/'new'/(model+'-export.json'));assert len(exported)==new*4 and all(len(x['chunks'])==1 for x in exported)
        verify_variants(stage)
        expected_native=(stage/'native-rows.json').read_bytes();expected_decisions=(stage/'decisions.json').read_bytes();expected_audit=(stage/'audit.json').read_bytes();expected_scores={p.name:p.read_bytes()for p in(stage/'scores').iterdir()}
        if label=='D':
            assert read(stage/'frozen-native.json')==[{**x,'id':'prior_'+x['id']}for x in read(root/'C/native-rows.json')]
            native=read(stage/'native-rows.json');boundaries=read(stage/'boundaries.json');assert len(native)==len(boundaries)==rows
            for x,b in zip(native,boundaries):
                left,sep,right=x['base']['text'].partition(' ');c=right[:1];letter=bool(c)and any(lo<=ord(c)<=hi for lo,hi in [(0x3040,0x30ff),(0x3400,0x4dbf),(0x4e00,0x9fff),(0xf900,0xfaff)])
                assert b=={'id':x['id'],'model':x['model'],'boundary':bool(sep)and left[-1:]in ['、','。']and ' 'not in right and letter}
            subprocess.run([sys.executable,str(stage/'assemble.py'),str(stage)],check=True,capture_output=True)
        else:
            cached=[]
            for name in ['first','positions','paper']:cached.extend({**x,'id':'old_'+name+'_'+x['id']}for x in read(root/'parent/raw'/name/'native-rows.json'))
            assert read(stage/'frozen-native.json')==cached
        subprocess.run([sys.executable,str(stage/'evaluate.py'),str(stage)],check=True,capture_output=True,env=os.environ|{'PYTHONUTF8':'1'})
        assert(stage/'native-rows.json').read_bytes()==expected_native;assert(stage/'decisions.json').read_bytes()==expected_decisions;assert(stage/'audit.json').read_bytes()==expected_audit
        for name,b in expected_scores.items():assert(stage/'scores'/name).read_bytes()==b
        print(label,count,'RGB geometries and18 score tables reproduced',flush=True)


def replay_extension(stage,parent,base,label,count,new):
    m=read(stage/'evaluation-manifest.json');assert len(m['inputs'])==count
    assert digest((parent/'evaluation-manifest.json').read_bytes())==m['parent_manifest_sha256']
    assert read(stage/'frozen-native.json')==read(parent/'native-rows.json')
    gm={x['id']:x for x in read(stage/'geometry.json')}
    for x in m['inputs']:
        for kind,folder,suffix in [('image','png','.png'),('rgb','rgb','.rgb'),('truth','truth','.txt')]:
            p=stage/'corpus'/folder/(x['id']+suffix)
            if x['cohort']!='new_control':
                p.parent.mkdir(parents=True,exist_ok=True);p.write_bytes((parent/'corpus'/folder/p.name).read_bytes())
            assert digest(p.read_bytes())==x[kind+'_sha256']
        body,gap,n=geometry((stage/'corpus/rgb'/(x['id']+'.rgb')).read_bytes(),x['width'],x['height'],True);g=gm[x['id']]
        assert (body,gap,n)==(g['body_height'],g['max_blank_gap'],g['components'])and g['supported']==(0<4*gap<=3*body)
    gc=read(stage/'geometry-command.json');assert gc['geometry_source_capture']==read(base/'geometry-command.json')and gc['exit_code']==0
    assert digest((stage/'prepare.py').read_bytes())==gc['generator_sha256'];assert digest((stage/'capture-spec.md').read_bytes())==gc['spec_sha256']
    rc=read(stage/'raw-command.json');assert digest((stage/'run_raw.py').read_bytes())==rc['driver_sha256']
    source=rc['raw_source_capture']
    if source.get('whole_roi'):
        for p,h in source['source_sha256'].items():assert digest((stage/'whole-roi-source'/p).read_bytes())==h
        proof=read(stage/'whole-roi-replay.json');assert len(proof['captured_rgb'])==new*4
        variants={x['id']:x for x in read(stage/'new/data/evaluation-manifest.json')['variants']}
        for x in proof['captured_rgb']:assert x['rgb_sha256']==variants[x['id']]['rgb_sha256']
        for model in ['30','50','100']:
            old=read(stage/'initial-chunked/new'/(model+'-export.json'));current={x['id']:x for x in read(stage/'new'/(model+'-export.json'))};compared=0
            for x in old:
                if len(x['chunks'])==1:assert x==current[x['id']];compared+=1
            pin=next(x for x in proof['models']if x['model']==model);assert pin['compared_identical_full_roi_rows']==compared
    else:assert rc['raw_binary_sha256']==read(base/'raw-command.json')['raw_binary_sha256']
    nm=read(stage/'new/data/evaluation-manifest.json');assert len(nm['inputs'])==new and digest((stage/'new/data/atlas.png').read_bytes())==nm['atlas_sha256']and digest((stage/'new/data/lines.json').read_bytes())==nm['lines_sha256']
    verify_variants(stage)
    for model in ['30','50','100']:
        assert read(stage/'new'/(model+'-command.json'))['exit_code']==0
        a=read(stage/'new'/(model+'-export.json'));assert len(a)==4*new and all(len(x['chunks'])==1 for x in a)
    native=read(stage/'native-rows.json');boundaries=read(stage/'boundaries.json');assert len(native)==len(boundaries)==count*3
    for x,b in zip(native,boundaries):
        left,sep,right=x['base']['text'].partition(' ');c=right[:1];letter=bool(c)and any(lo<=ord(c)<=hi for lo,hi in [(0x3040,0x30ff),(0x3400,0x4dbf),(0x4e00,0x9fff),(0xf900,0xfaff)])
        assert b=={'id':x['id'],'model':x['model'],'boundary':bool(sep)and left[-1:]in ['、','。']and ' 'not in right and letter}
    # Exact D selection function: controls do not silently change the hypothesis.
    fn=lambda p:p.read_text(encoding='utf-8').split('def japanese')[1].split('native=json')[0]
    assert fn(stage/'evaluate.py')==fn(base/'evaluate.py')
    expected={name:(stage/name).read_bytes()for name in ['native-rows.json','decisions.json','audit.json']};tables={p.name:p.read_bytes()for p in(stage/'scores').iterdir()}
    for driver in ['assemble.py','evaluate.py']:subprocess.run([sys.executable,str(stage/driver),str(stage)],check=True,capture_output=True,env=os.environ|{'PYTHONUTF8':'1'})
    for name,b in expected.items():assert(stage/name).read_bytes()==b
    for name,b in tables.items():assert(stage/'scores'/name).read_bytes()==b
    print(label,count,'RGB geometries and18 score tables reproduced',flush=True)


def replay_advance(stage,parent):
    m=read(stage/'evaluation-manifest.json');assert len(m['inputs'])==3446
    assert digest((parent/'evaluation-manifest.json').read_bytes())==m['parent_manifest_sha256']
    assert read(stage/'frozen-native.json')==read(parent/'native-rows.json')
    width={x['id']:x for x in read(stage/'geometry-width.json')};advance={x['id']:x for x in read(stage/'geometry-advance.json')}
    for x in m['inputs']:
        for kind,folder,suffix in [('image','png','.png'),('rgb','rgb','.rgb'),('truth','truth','.txt')]:
            p=stage/'corpus'/folder/(x['id']+suffix)
            if x['cohort']!='new_control':p.parent.mkdir(parents=True,exist_ok=True);p.write_bytes((parent/'corpus'/folder/p.name).read_bytes())
            assert digest(p.read_bytes())==x[kind+'_sha256']
        body,gap,_,span,ww,anchors=geometry((stage/'corpus/rgb'/(x['id']+'.rgb')).read_bytes(),x['width'],x['height'],True,True);w=width[x['id']];a=advance[x['id']]
        assert(body,gap,ww,anchors)==(w['body_height'],w['max_blank_gap'],w['body_width'],w['anchors'])
        assert w['supported']==(0<4*gap<=3*body and ww is not None and 4*gap<=3*ww)
        assert(body,gap,span)==(a['body_height'],a['max_blank_gap'],a['foreground_width'])
        for n in range(6,13):assert a['support_by_length'][str(n)]==(0<4*gap<=3*body and 4*gap*n<=3*span)
    for meta,folder,field in [('geometry-command.json','geometry-source','sha256'),('advance-command.json','advance-source','sha256'),('production-replay-command.json','production-source','source_sha256')]:
        for p,h in read(stage/meta)[field].items():assert digest((stage/folder/p).read_bytes())==h
    source=read(stage/'parent-raw-capture.json');assert source['whole_roi']
    for p,h in source['source_sha256'].items():assert digest((stage/'whole-roi-source'/p).read_bytes())==h
    verify_variants(stage);variants={x['id']:x for x in read(stage/'new/data/evaluation-manifest.json')['variants']};assert len(variants)==4608
    proof=read(stage/'shared-raw-replay.json');assert len(proof['captured_rgb'])==4608
    for x in proof['captured_rgb']:assert x['rgb_sha256']==variants[x['id']]['rgb_sha256']
    for model in ['30','50','100']:
        assert read(stage/'new'/(model+'-command.json'))['exit_code']==0
        rows=read(stage/'new'/(model+'-export.json'));assert len(rows)==4608 and all(len(x['chunks'])==1 for x in rows)
    expected={name:(stage/name).read_bytes()for name in ['native-rows.json','decisions-width.json','decisions-advance.json','audit-width.json','audit-advance.json']}
    tables={directory:{p.name:p.read_bytes()for p in(stage/directory).iterdir()}for directory in ['scores-width','scores-advance']}
    for driver in ['assemble.py','evaluate-width.py','evaluate-advance.py']:subprocess.run([sys.executable,str(stage/driver),str(stage)],check=True,capture_output=True,env=os.environ|{'PYTHONUTF8':'1'})
    for name,b in expected.items():assert(stage/name).read_bytes()==b
    for directory,values in tables.items():
        for name,b in values.items():assert(stage/directory/name).read_bytes()==b
    f32=lambda x:struct.unpack('<f',struct.pack('<f',x))[0]
    native={(x['id'],x['model']):x for x in read(stage/'native-rows.json')};decisions={(x['id'],x['model']):x for x in read(stage/'decisions-advance.json')};images={x['id']:x for x in m['inputs']};records=read(stage/'production-replay/records.json');assert len(records)==10338
    for row in records:
        key=(row['id'],row['model']);x=native[key];d=decisions[key];im=images[row['id']];modelw={'30':256,'50':384,'100':768}[row['model']]
        assert 5*im['height']<=4*im['width']
        assert row['raw']['text']==x['base']['text']and f32(row['raw']['mean_confidence'])==x['base']['mean_confidence']
        cap=(im['width']+4)*24<=im['height']*modelw;n=len(x['base']['text'].replace(' ',''))
        eligible=cap and all(d['checks'][k]for k in ['single_internal_space','japanese_only','length6to12','punctuation_boundary'])and advance[row['id']]['support_by_length'].get(str(n),False)
        calls=0;selected=False
        if eligible:
            for index,alt in enumerate(x['candidates']):
                calls+=1
                if alt['text']!=x['base']['text'].replace(' ','')or not math.isfinite(alt['mean_confidence'])or not .98<=alt['mean_confidence']<=1 or(index==0 and alt['mean_confidence']<=x['base']['mean_confidence']):break
            else:selected=True
        assert(row['selected'],row['calls'])==(selected,calls)
        chosen=x['candidates'][0]if selected else x['base'];assert row['result']['text']==chosen['text']and f32(row['result']['mean_confidence'])==chosen['mean_confidence']
        for policy in ['single','fragment']:
            for phase in ['before','after']:
                p=stage/'production-replay'/row['model']/policy/phase/(row['id']+'.txt');p.parent.mkdir(parents=True,exist_ok=True);p.write_bytes(row['policies'][policy][phase].encode('utf-8'))
    expected_audit=(stage/'production-replay/audit.json').read_bytes();scores=stage/'production-replay/scores';expected_scores={p.name:p.read_bytes()for p in scores.iterdir()};scores.rename(stage/'production-replay/expected-scores');command=(stage/'production-score-command.json').read_bytes()
    code=(stage/'score_production_replay.py').read_text(encoding='utf-8').replace("r=Path('tmp/punct-width')","r=Path(sys.argv[1])").replace("scorer=Path('tools/eval_ocr_metrics.py')","scorer=Path(sys.argv[2])");driver=stage/'replay-production-scores.py';driver.write_text(code,encoding='utf-8');subprocess.run([sys.executable,str(driver),str(stage),str(stage/'source/eval_ocr_metrics.py')],check=True,env=os.environ|{'PYTHONUTF8':'1'})
    assert(stage/'production-replay/audit.json').read_bytes()==expected_audit
    for name,b in expected_scores.items():assert(scores/name).read_bytes()==b
    (stage/'production-score-command.json').write_bytes(command)
    print('E/F3446 RGB geometries,36 diagnostic tables and36 production policy tables reproduced;10338 selections checked',flush=True)

def materialize(root):
    spec=read(root/'materialize.json')
    for path in spec['record_directories']:
        d=root/safe(path)
        for x in read(d/'results.json'):(d/(x['id']+'.txt')).write_bytes(x['text'].encode('utf-8'))
    for p,h in spec.get('actual_body_sha256',{}).items():assert digest((root/safe(p)).read_bytes())==h

def replay_validation(stage,advance):
    materialize(stage)
    manifest=read(stage/'native-manifest.json');assert len(manifest['inputs'])==158
    parent=read(advance/'evaluation-manifest.json')
    assert digest((advance/'evaluation-manifest.json').read_bytes())==manifest['parent_manifest_sha256']
    assert digest((advance/'production-replay/records.json').read_bytes())==manifest['selection_records_sha256']
    for x in manifest['inputs']:
        for kind,suffix in [('image','.png'),('rgb','.rgb'),('truth','.txt')]:
            b=(stage/'native-inputs'/kind/(x['id']+suffix)).read_bytes();assert digest(b)==x[kind+'_sha256']
    capture=read(stage/'capture-command.json')
    for p,h in capture['source']['sha256'].items():assert digest((stage/'source'/p).read_bytes())==h
    assert digest((stage/'native-manifest.json').read_bytes())==capture['native_manifest_sha256']
    assert digest((stage/'page-after/evaluation-manifest.json').read_bytes())==capture['page_manifest_sha256']
    records={(x['id'],x['model']):x for x in read(advance/'production-replay/records.json')}
    f32=lambda x:struct.unpack('<f',struct.pack('<f',x))[0]
    checked=0
    for phase in ['before','after']:
        for model in ['30','50','100']:
            assert read(stage/(phase+'-native'+model+'-command.json'))['exit_code']==0
            for route in (stage/('native-'+phase)/model).iterdir():
                actual=read(route/'results.json');assert len(actual)==158
                for x in actual:
                    old=records[x['id'],model]
                    if route.name=='raw_session':text=old['raw']['text'];mean=old['raw']['mean_confidence']
                    else:
                        policy='fragment'if route.name=='fragment_horizontal_batch'else'single'
                        text=old['policies'][policy][phase];mean=old['raw'if phase=='before'else'result']['mean_confidence']
                    assert x['text']==text and f32(x['mean_confidence'])==f32(mean)
                    checked+=1
    assert checked==7584
    for x in read(stage/'page-after/evaluation-manifest.json')['inputs']:
        assert digest((stage/'images'/(x['id']+Path(x['image']).suffix)).read_bytes())==x['image_sha256']
        assert digest((stage/'page-truth'/(x['id']+'.txt')).read_bytes())==x['truth_sha256']
    for kind in ['native','page']:
        expected=(stage/(kind+'-audit.json')).read_bytes();tables={p.name:p.read_bytes()for p in(stage/(kind+'-scores')).iterdir()}
        subprocess.run([sys.executable,str(stage/'score_matrix.py'),str(stage),kind,str(stage/(kind+'-truth')),str(stage/'source/eval_ocr_metrics.py')],check=True,env=os.environ|{'PYTHONUTF8':'1'})
        assert(stage/(kind+'-audit.json')).read_bytes()==expected
        for name,b in tables.items():assert(stage/(kind+'-scores')/name).read_bytes()==b
        print('F',kind,len(tables),'tables and audit reproduced',flush=True)
    line=read(stage/'line-cli-audit.json');assert line['processes']==14 and line['exact_recoveries']==3 and line['negative_pairs_unchanged']==4
    for x in line['records']:
        name=x['phase']+'-'+x['model']+'-'+x['id'];body=(stage/'line-cli'/(name+'.stdout')).read_bytes()
        assert x['exit_code']==0 and body[-1:]==b'\n'and body[:-1]==x['text'].encode('utf-8')
    print('F7584 real text/f32 records match production replay;14 CLI witnesses checked',flush=True)

def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('destination',type=Path);p.add_argument('--raw-only',action='store_true');args=p.parse_args();root=args.destination.resolve();root.mkdir(parents=True);here=Path(__file__).resolve().parent
    extract(here,'raw',root/'raw');replay_raw(root/'raw')
    parent=root/'raw/D'
    for name,count,new in [('noise',1718,360),('aspect',2294,576)]:
        if(here/(name+'.zip')).exists():
            stage=root/name;extract(here,name,stage);replay_extension(stage,parent,root/'raw/D',name,count,new);parent=stage
    if(here/'advance.zip').exists():
        stage=root/'advance';extract(here,'advance',stage);replay_advance(stage,parent)
    if args.raw_only:return
    for name in ['native','page']:
        stage=root/name;extract(here,name,stage);materialize(stage);expected=(stage/(name+'-audit.json')).read_bytes();tables={p.name:p.read_bytes()for p in(stage/(name+'-scores')).iterdir()}
        data=root/'raw/D'if name=='native'else stage/'page-truth';scorer=root/'raw/D/source/eval_ocr_metrics.py'
        subprocess.run([sys.executable,str(stage/'score_matrix.py'),str(stage),name,str(data),str(scorer)],check=True,env=os.environ|{'PYTHONUTF8':'1'})
        assert(stage/(name+'-audit.json')).read_bytes()==expected
        for key,b in tables.items():assert(stage/(name+'-scores')/key).read_bytes()==b
        print(name,len(tables),'tables and audit reproduced',flush=True)
    if(here/'validation.zip').exists():
        stage=root/'validation';extract(here,'validation',stage);replay_validation(stage,root/'advance')
    folders=['raw/C/scores','raw/D/scores','noise/scores','aspect/scores','advance/scores-width','advance/scores-advance','advance/production-replay/scores','native/native-scores','page/page-scores','validation/native-scores','validation/page-scores']
    tables={str(p.relative_to(root)):digest(p.read_bytes())for folder in folders for p in(root/folder).iterdir()if p.suffix in ['.csv','.md']}
    assert len(tables)==528 and sum(p.endswith('.csv')for p in tables)==264
    archives={name:read(here/(name+'.json'))['sha256']for name in ['raw','noise','aspect','advance','native','page','validation']}
    report={'archive_sha256':archives,'verified_table_files':len(tables),'table_sha256':tables,'all_archive_member_sha256_checked':True,'all_original_body_sha256_checked':True,'replayer_sha256':digest(Path(__file__).read_bytes())}
    (root/'verification.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8')
    print('all7 archives and528 CSV/Markdown files reproduced',flush=True)

if __name__=='__main__':main()
