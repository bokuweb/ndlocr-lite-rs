#!/usr/bin/env python3
"""Verify the bundle; replay12 primary50 tables and54 raw-space tables, four audits and40 controls."""
from pathlib import Path
import argparse,csv,hashlib,json,os,shutil,subprocess,sys,zipfile

def canonical(path):
    return path.read_bytes().replace(b'\r\n',b'\n')

def main():
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('destination',type=Path);root=parser.parse_args().destination.resolve();root.mkdir(parents=True,exist_ok=False);here=Path(__file__).resolve().parent;index=json.loads((here/'evidence.json').read_text(encoding='utf-8'));sha=lambda b:hashlib.sha256(b).hexdigest();archive=here/'evidence.zip';assert sha(archive.read_bytes())==index['sha256']
    with zipfile.ZipFile(archive)as z:
        assert len(z.namelist())==len(set(z.namelist()))==index['member_count'];assert set(z.namelist())==set(index['members'])
        for key,s in index['members'].items():
            p=Path(key);assert not p.is_absolute()and '..'not in p.parts;data=z.read(key);assert sha(data)==s,key;target=root/p;target.parent.mkdir(parents=True,exist_ok=True);target.write_bytes(data)
    page=root/'page';m=json.loads((page/'evaluation-manifest.json').read_text(encoding='utf-8'))
    for row in m['inputs']:
        for kind in ['image','truth']:
            p=page/('images'if kind=='image'else'truth')/(row['id']+(Path(row[kind]).suffix if kind=='image'else'.txt'));assert sha(p.read_bytes())==row[kind+'_sha256']
    controls=json.loads((page/'control-reference.json').read_text(encoding='utf-8'))['records'];assert len(controls)==40
    for x in controls:
        d=page/'controls'/(x['phase']+x['model'])/x['mode'];rows={r['id']:r for r in json.loads((d/'results.json').read_text(encoding='utf-8'))};v=x['record'];actual=rows[v['id']];assert actual['page_index']==0;assert {k:actual[k]for k in ['id','text','lines']}==v;assert (d/(v['id']+'.txt')).read_bytes()==v['text'].encode()
    scorer=root/'source/eval_ocr_metrics.py';output=root/'replayed/page';output.mkdir(parents=True);audit={'images':287,'comparisons_per_metric':1722,'changes':[],'char_regressions':[],'word_regressions':[],'metadata_changes':[]};tables=0
    for suffix in ['eval','default']:
        mode='runtime50_'+suffix;before=page/'before'/mode;after=page/'after'/mode;b={x['id']:x for x in json.loads((before/'results.json').read_text(encoding='utf-8'))};a={x['id']:x for x in json.loads((after/'results.json').read_text(encoding='utf-8'))};assert len(a)==len(b)==287
        for id in b:
            if b[id]!=a[id]:audit['metadata_changes'].append([mode,id])
        for norm in ['none','width','strict']:
            table=output/(mode+'-'+norm+'.csv');report=table.with_suffix('.md');subprocess.run([sys.executable,str(scorer),'--truth-dir',str(page/'truth'),'--system','current='+str(before),'--system','candidate='+str(after),'--normalize',norm,'--require-all','--output-csv',str(table),'--output-md',str(report)],check=True,capture_output=True);table.write_bytes(canonical(table));report.write_bytes((report.read_text(encoding='utf-8').rstrip()+'\n').encode())
            for p in [table,report]:assert p.read_bytes()==(root/'expected/page'/p.name).read_bytes(),p;tables+=1
            records=list(csv.DictReader(table.open(encoding='utf-8',newline='')));bs={x['file']:x for x in records if x['system']=='current'};aa={x['file']:x for x in records if x['system']=='candidate'};assert len(bs)==len(aa)==287
            for id in bs:
                bc,ac=int(bs[id]['char_errors']),int(aa[id]['char_errors']);bw,aw=int(bs[id]['word_errors']),int(aa[id]['word_errors'])
                if (before/id).read_bytes()!=(after/id).read_bytes():audit['changes'].append([mode,norm,id,bc,ac,bw,aw])
                if ac>bc:audit['char_regressions'].append([mode,norm,id,bc,ac])
                if aw>bw:audit['word_regressions'].append([mode,norm,id,bw,aw])
    data=(json.dumps(audit,indent=2)+'\n').encode();assert data==(page/'audit.json').read_bytes();(output/'audit.json').write_bytes(data);prior=json.loads((page/'parent-audit.json').read_text(encoding='utf-8'));combined=json.loads((page/'combined-summary.json').read_text(encoding='utf-8'));assert prior['comparisons']+audit['comparisons_per_metric']==combined['combined_comparisons_per_metric']==6888;assert not prior['char_regressions']and not prior['word_regressions']and not audit['char_regressions']and not audit['word_regressions'];print('primary50:1722 comparisons;40 controls;12 tables and audit reproduced',flush=True)
    for stage in ['first','positions','paper']:
        source=root/'raw'/stage;target=root/'replayed'/stage;shutil.copytree(source,target);expected=[p for p in source.rglob('*')if p.is_file()and(p.relative_to(source).parts[0]in ['scores','predictions']or p.name in ['audit.json','decisions.json','native-rows.json'])];subprocess.run([sys.executable,str(target/'evaluate.py'),str(target)],check=True,capture_output=True,env=os.environ|{'PYTHONUTF8':'1'})
        for p in expected:
            actual=target/p.relative_to(source)
            if p.suffix=='.json':actual.write_bytes(canonical(actual))
            assert actual.read_bytes()==p.read_bytes(),actual
        tables+=len(list((source/'scores').glob('*.csv')))+len(list((source/'scores').glob('*.md')));print(stage,len(expected),'derived outputs reproduced',flush=True)
    assert tables==66;print(index['member_count'],'members verified;',tables,'byte-identical tables,4 audits and40 controls',flush=True)

if __name__=='__main__':main()
