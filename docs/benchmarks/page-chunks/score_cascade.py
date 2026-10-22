from pathlib import Path
import json,csv,subprocess
r=Path('tmp/page-chunk-probe');rows_all=[];reg=[]
for mode in ['deim','bands']:
 for norm in ['none','width','strict']:
  default=Path('docs/benchmarks/url-hyphen')/mode/'after'
  subprocess.run(['python3','tools/eval_ocr_metrics.py','--truth-dir',str(r/'truth'),'--system',f'cascade={default}','--system',f'fixed100={r/mode/"before"}','--normalize',norm,'--require-all','--output-csv',str(r/f'cascade-{mode}-{norm}.csv'),'--output-md',str(r/f'cascade-{mode}-{norm}.md')],check=True,capture_output=True)
  rows=list(csv.DictReader(open(r/f'cascade-{mode}-{norm}.csv')))
  b={x['file']:int(x['char_errors']) for x in rows if x['system']=='cascade'};a={x['file']:int(x['char_errors']) for x in rows if x['system']=='fixed100'};assert len(a)==len(b)==20
  for name in a:
   if a[name]!=b[name]:rows_all.append([mode,norm,name,b[name],a[name]])
   if a[name]>b[name]:reg.append([mode,norm,name,b[name],a[name]])
(r/'cascade-audit.json').write_text(json.dumps({'comparisons':120,'changes':rows_all,'regressions':reg},indent=2)+'\n')
print('cascade -> all100 regressions',len(reg));print('raw changes',[x for x in rows_all if x[1]=='none'])
