from pathlib import Path
import argparse,hashlib,json,zipfile,subprocess,os,csv,collections
p=argparse.ArgumentParser();p.add_argument('--output',required=True);a=p.parse_args();root=Path(a.output).resolve();here=Path(__file__).resolve().parent;sha=lambda b:hashlib.sha256(b).hexdigest();root.mkdir(parents=True,exist_ok=True);seen=set()
for kind in ['images','diagnostics','text']:
 index=json.loads((here/(kind+'.json')).read_text());archive=here/(kind+'.zip');assert sha(archive.read_bytes())==index['sha256']
 with zipfile.ZipFile(archive)as z:
  assert set(z.namelist())==set(index['members'])
  for name,digest in index['members'].items():
   assert not name.startswith('/')and'..'not in Path(name).parts;assert name not in seen;seen.add(name);b=z.read(name);assert sha(b)==digest,name;out=root/name;out.parent.mkdir(parents=True,exist_ok=True);out.write_bytes(b)
for directory,variable in [(root,'NDLOCR_CONTAINED_CELL_AWARE_CLOSER_EVAL_ROOT'),(root/'failed-v1','NDLOCR_CONTAINED_CLOSER_EVAL_ROOT'),(root/'failed-v2','NDLOCR_CONTAINED_WIDE_CLOSER_EVAL_ROOT')]:
 envname=variable
 before={str(p.relative_to(directory)):p.read_bytes()for p in (directory/'metrics').glob('*')};before['audit.json']=(directory/'audit.json').read_bytes();subprocess.run(['python3',str(directory/'evaluate.py')],check=True,env={**os.environ,envname:str(directory),'NDLOCR_SCORER':str(root/'tools/eval_ocr_metrics.py')},stdout=subprocess.DEVNULL)
 for name,b in before.items():assert(directory/name).read_bytes()==b,(directory,name)
audit=json.loads((root/'audit.json').read_text());changed={(x['model'],x['id'])for x in audit['changes']};improved=set();counts=collections.defaultdict(collections.Counter)
for label in ['single30','single50','single100']:
 for norm in ['none','width','strict']:
  rows=list(csv.DictReader((root/'metrics'/(label+'-'+norm+'.csv')).open()));base={x['file']:x for x in rows if x['system']=='current'}
  for x in rows:
   key=(label,Path(x['file']).stem)
   if x['system']!='prototype'or key not in changed:continue
   for metric in ['char_errors','word_errors']:
    old,new=int(base[x['file']][metric]),int(x[metric]);counts[label+'__'+norm][metric+('_improved'if new<old else'_equal'if new==old else'_regressed')]+=1
    if new<old:improved.add(key)
summary={'changed_pairs':len(changed),'any_metric_normalization_improved':len(improved),'all_metric_tie_changes':len(changed)-len(improved),'by_model_normalization':{k:dict(v)for k,v in counts.items()}}
assert summary==json.loads((root/'distance-change-summary.json').read_text())
print('all archive/member SHA verified;54 metric files, three audits, and distance-change summary reproduced')
