from pathlib import Path
import json,hashlib,os,subprocess,concurrent.futures,time,sys
from single_transport import recognition_content
r=Path(os.environ.get('NDLOCR_REPEAT_EVAL_ROOT','tmp/repeated-glyph'));phase=sys.argv[1];m=json.loads((r/'evaluation-manifest.json').read_text());rows=[x for x in m['inputs']if x['cohort']in['repeated_glyph','long_line','dense_line','dense_crop','page_band_fallback']and not x['id'].startswith('fallback_two_rows_')]
assert len(rows)==263,(len(rows),{x['cohort']for x in m['inputs']});sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest();truth=r/'single-truth';truth.mkdir(exist_ok=True)
if phase!='score':
 for row in rows:(truth/(row['id']+'.txt')).write_bytes(Path(row['truth']).read_bytes())
models=[('single30','models/parseq-ndl-24x256-30-tiny-189epoch-tegaki3-r8data-202604.onnx'),('single50','models/parseq-ndl-24x384-50-tiny-300epoch-tegaki3-r8data-202604.onnx'),('single100','models/parseq-ndl-24x768-100-tiny-153epoch-tegaki3-r8data-202604.onnx')]
if phase in ['before','after']:
 binary=r/('baseline-ndlocr'if phase=='before'else'candidate-ndlocr')
 if phase=='before':assert sha(binary)==m['baseline_cli_binary_sha256']
 def run(job):
  row,(mode,model)=job;assert sha(row['image'])==row['image_sha256']and sha(row['truth'])==row['truth_sha256'];out=r/phase/mode/(row['id']+'.txt');out.parent.mkdir(parents=True,exist_ok=True);argv=[str(binary.resolve()),'recognize','--model',model,'--image',row['image'],'--charset','models/NDLmoji.yaml'];start=time.monotonic();p=subprocess.run(argv,capture_output=True,env={**os.environ,'NDLOCR_INTRA_THREADS':'1'});out.with_suffix('.stdout').write_bytes(p.stdout);out.with_suffix('.stderr').write_bytes(p.stderr);assert p.returncode==0,(argv,p.stderr);out.write_bytes(recognition_content(p.stdout));return {'argv':argv,'exit_code':p.returncode,'elapsed_seconds':time.monotonic()-start}
 with concurrent.futures.ThreadPoolExecutor(max_workers=3)as pool:jobs=list(pool.map(run,[(row,model)for row in rows for model in models]))
 (r/(phase+'-single-runs.json')).write_text(json.dumps({'binary_sha256':sha(binary),'runs':jobs},indent=2)+'\n');print(phase,'single CLI complete',len(jobs),flush=True)
if phase=='score':
 import csv
 changes=[];reg=[];word=[]
 for mode,_ in models:
  for norm in ['none','width','strict']:
   stem=mode+'-'+norm;subprocess.run(['python3','tools/eval_ocr_metrics.py','--truth-dir',str(truth),'--system','before='+str(r/'before'/mode),'--system','after='+str(r/'after'/mode),'--normalize',norm,'--require-all','--output-csv',str(r/(stem+'.csv')),'--output-md',str(r/(stem+'.md'))],check=True,capture_output=True)
   p=r/(stem+'.csv');p.write_bytes(p.read_bytes().replace(b'\r\n',b'\n'));p=r/(stem+'.md');p.write_text(p.read_text().rstrip()+'\n');scored=list(csv.DictReader((r/(stem+'.csv')).open()));b={x['file']:x for x in scored if x['system']=='before'};a={x['file']:x for x in scored if x['system']=='after'};assert len(a)==len(b)==263
   for name in a:
    old=int(b[name]['char_errors']);new=int(a[name]['char_errors']);row=[mode,norm,name,old,new]
    if old!=new:changes.append(row)
    if new>old:reg.append(row)
    old=int(b[name]['word_errors']);new=int(a[name]['word_errors'])
    if new>old:word.append([mode,norm,name,old,new])
 (r/'single-audit.json').write_text(json.dumps({'comparisons':263*3*3,'changes':changes,'regressions':reg,'word_regressions':word},indent=2)+'\n');print('single regressions',reg,flush=True)
