from pathlib import Path
import concurrent.futures,json,os,subprocess,time,hashlib,csv
import sys
root=Path(os.environ.get('NDLOCR_BAND_EVAL_ROOT','tmp/page-band-fallback'));phase=sys.argv[1]
manifest=json.loads((root/'evaluation-manifest.json').read_text());rows=manifest['inputs'];modes=['deim','bands','runtime30_eval','runtime30_default','runtime100_eval','runtime100_default'];sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest()
if phase in ['before','after']:
 binary=Path('tmp/dense-crop/candidate-ndlocr')if phase=='before'else root/'candidate-ndlocr'
 if phase=='before':assert sha(binary)==manifest['before_cli_binary_sha256']
 def run(job):
  row,mode=job;out=root/phase/mode/(row['id']+'.txt');out.parent.mkdir(parents=True,exist_ok=True)
  assert sha(row['image'])==row['image_sha256']and sha(row['truth'])==row['truth_sha256']
  if phase=='before'and row['cohort']!='page_band_fallback':assert out.exists();return {'reused':str(out)}
  argv=[str(binary.resolve()),'recognize-page','--image',row['image'],'--charset','models/NDLmoji.yaml','--use-deim-detection',str(mode=='deim').lower(),'--output-txt',str(out)];start=time.monotonic();p=subprocess.run(argv,capture_output=True,env={**os.environ,'NDLOCR_INTRA_THREADS':'1'});out.with_suffix('.stderr').write_bytes(p.stderr);assert p.returncode==0,(argv,p.stderr)
  return {'argv':argv,'exit_code':p.returncode,'elapsed_seconds':time.monotonic()-start}
 with concurrent.futures.ThreadPoolExecutor(max_workers=3)as pool:records=list(pool.map(run,[(row,mode)for row in rows for mode in ['deim','bands']]))
 (root/(phase+'-runs.json')).write_text(json.dumps({'binary_sha256':sha(binary),'runs':records},indent=2)+'\n');print(phase,'CLI complete',len(records),flush=True)
if phase=='score':
 changes=[];reg=[];word=[]
 for mode in modes:
  for norm in ['none','width','strict']:
   stem=mode+'-'+norm
   subprocess.run(['python3','tools/eval_ocr_metrics.py','--truth-dir',str(root/'truth'),'--system','before='+str(root/'before'/mode),'--system','after='+str(root/'after'/mode),'--normalize',norm,'--require-all','--output-csv',str(root/(stem+'.csv')),'--output-md',str(root/(stem+'.md'))],check=True,capture_output=True)
   p=root/(stem+'.csv');p.write_bytes(p.read_bytes().replace(b'\r\n',b'\n'));p=root/(stem+'.md');p.write_text(p.read_text().rstrip()+'\n')
   scored=list(csv.DictReader((root/(stem+'.csv')).open()));b={x['file']:x for x in scored if x['system']=='before'};a={x['file']:x for x in scored if x['system']=='after'};assert len(a)==len(b)==127
   for name in a:
    old=int(b[name]['char_errors']);new=int(a[name]['char_errors']);pair=[mode,norm,name,old,new]
    if old!=new:changes.append(pair)
    if new>old:reg.append(pair)
    old=int(b[name]['word_errors']);new=int(a[name]['word_errors'])
    if new>old:word.append([mode,norm,name,old,new])
 (root/'audit.json').write_text(json.dumps({'comparisons':127*6*3,'changes':changes,'regressions':reg,'word_regressions':word},indent=2)+'\n');print('regressions',reg,flush=True)
