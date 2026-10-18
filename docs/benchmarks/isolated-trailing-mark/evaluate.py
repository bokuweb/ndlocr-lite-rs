from pathlib import Path
import concurrent.futures,hashlib,json,os,shutil,subprocess,sys,time,csv
r=Path(os.environ.get('NDLOCR_DENSE_EVAL_ROOT','tmp/dense-crop')); phase=sys.argv[1]; old=Path('docs/benchmarks/dense-line'); baseline=Path('tmp/dense-line/candidate-ndlocr'); binary=baseline if phase=='before' else r/'candidate-ndlocr'
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest()
if phase=='before':
 oldm=json.loads((old/'evaluation-manifest.json').read_text(encoding="utf-8")); assert sha(baseline)==oldm['after_binary_sha256']
 inputs=oldm['inputs']; new=json.loads(Path('tests/fixtures/dense_crop/manifest.json').read_text(encoding="utf-8"))['rows']; assert len(new)==24
 for x in new:
  assert sha(x['image'])==x['image_sha256']
  inputs.append({'id':'crop_'+x['id'],'image':x['image'],'truth':'tests/fixtures/dense_crop/truth/'+x['id']+'.txt','cohort':'dense_crop'})
 assert len(inputs)==103
 (r/'truth').mkdir(exist_ok=True)
 for x in inputs:
  shutil.copyfile(x['truth'],r/'truth'/(x['id']+'.txt'));x['image_sha256']=sha(x['image']);x['truth_sha256']=sha(x['truth'])
 (r/'evaluation-manifest.json').write_text(json.dumps({'baseline_commit':'e9b1bd300d2a7905ad1966ec355d4b36cdca9883','inputs':inputs,'before_binary_sha256':sha(baseline),'new_manifest_sha256':sha('tests/fixtures/dense_crop/manifest.json')},ensure_ascii=False,indent=2)+'\n')
else: inputs=json.loads((r/'evaluation-manifest.json').read_text(encoding="utf-8"))['inputs']
def run(job):
 x,mode=job; dest=r/phase/mode;dest.mkdir(parents=True,exist_ok=True);out=dest/(x['id']+'.txt')
 assert sha(x['image'])==x['image_sha256'] and sha(x['truth'])==x['truth_sha256']
 if phase=='before' and x['cohort']!='dense_crop':
  shutil.copyfile(old/'after'/mode/out.name,out);return {'reused':str(old/'after'/mode/out.name)}
 argv=[str(binary.resolve()),'recognize-page','--image',x['image'],'--charset','models/NDLmoji.yaml','--use-deim-detection',str(mode=='deim').lower(),'--output-txt',str(out)]
 start=time.monotonic();p=subprocess.run(argv,capture_output=True,env={**os.environ,'NDLOCR_INTRA_THREADS':'1'});(dest/(x['id']+'.stderr')).write_bytes(p.stderr);assert p.returncode==0,(argv,p.stderr)
 return {'argv':argv,'exit_code':p.returncode,'elapsed_seconds':time.monotonic()-start}
if phase!='score':
 with concurrent.futures.ThreadPoolExecutor(max_workers=3) as pool:records=list(pool.map(run,[(x,m) for x in inputs for m in ['deim','bands']]))
 (r/(phase+'-runs.json')).write_text(json.dumps({'binary_sha256':sha(binary),'runs':records},indent=2)+'\n');print(phase,'completed',len(records),flush=True)
if phase in ['after','score']:
 changes=[];reg=[]
 for mode in ['deim','bands','runtime30','runtime100']:
  for norm in ['none','width','strict']:
   stem=mode+'-'+norm
   subprocess.run(['python3','tools/eval_ocr_metrics.py','--truth-dir',str(r/'truth'),'--system','before='+str(r/'before'/mode),'--system','after='+str(r/'after'/mode),'--normalize',norm,'--require-all','--output-csv',str(r/(stem+'.csv')),'--output-md',str(r/(stem+'.md'))],check=True,capture_output=True)
   report=r/(stem+'.csv');report.write_bytes(report.read_bytes().replace(b'\r\n',b'\n'))
   report=r/(stem+'.md');report.write_text(report.read_text(encoding="utf-8").rstrip()+'\n')
   rows=list(csv.DictReader(open(r/(stem+'.csv'))));b={x['file']:int(x['char_errors']) for x in rows if x['system']=='before'};a={x['file']:int(x['char_errors']) for x in rows if x['system']=='after'};assert len(a)==len(b)==103
   for name in a:
    if a[name]!=b[name]:changes.append([mode,norm,name,b[name],a[name]])
    if a[name]>b[name]:reg.append([mode,norm,name,b[name],a[name]])
 (r/'audit.json').write_text(json.dumps({'comparisons':103*4*3,'changes':changes,'regressions':reg},indent=2)+'\n');print('regressions',reg,flush=True)
