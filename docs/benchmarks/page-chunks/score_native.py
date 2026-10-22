from pathlib import Path
import csv,json,subprocess
r=Path('tmp/page-chunk-probe/native');replays=[]
for mode in ['deim','bands']:
 for p in sorted((r/mode/'before').glob('*.txt')):
  q=r.parent/'cli'/mode/p.name
  assert p.read_bytes()==q.read_bytes(),(p,q)
  replays.append(str(q))
assert len(replays)==40
changes=[];regressions=[]
for mode in ['deim','bands']:
 for norm in ['none','width','strict']:
  subprocess.run(['python3','tools/eval_ocr_metrics.py','--truth-dir',str(r.parent/'truth'),'--system',f'before={r/mode/"before"}','--system',f'after={r/mode/"after"}','--normalize',norm,'--require-all','--output-csv',str(r/f'{mode}-{norm}.csv'),'--output-md',str(r/f'{mode}-{norm}.md')],check=True,capture_output=True)
  rows=list(csv.DictReader(open(r/f'{mode}-{norm}.csv')))
  b={x['file']:int(x['char_errors']) for x in rows if x['system']=='before'};a={x['file']:int(x['char_errors']) for x in rows if x['system']=='after'}
  assert len(b)==len(a)==20
  for name in b:
   if a[name]!=b[name]:changes.append([mode,norm,name,b[name],a[name]])
   if a[name]>b[name]:regressions.append([mode,norm,name,b[name],a[name]])
(r/'audit.json').write_text(json.dumps({'cli_replays':replays,'comparisons':120,'changes':changes,'regressions':regressions},indent=2)+'\n')
print('CLI parity',len(replays),'regressions',len(regressions))
print('originals',[x for x in changes if x[2] in ['jma_climate_2025.txt','jma_kobe_notice.txt','ndl_accessibility_p2.txt']])
