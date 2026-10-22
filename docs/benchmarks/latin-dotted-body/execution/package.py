from pathlib import Path
import json,hashlib,zipfile,zlib,os,shutil
r=Path('tmp/latin-dotted-body');out=Path('docs/benchmarks/latin-dotted-body');sha=lambda b:hashlib.sha256(b).hexdigest()
def main():
 out.mkdir(parents=True,exist_ok=True);files={}
 def add(key,p):
  p=Path(p)
  if key in files:assert files[key].read_bytes()==p.read_bytes()
  else:files[key]=p
 for p in [*Path('src').rglob('*.rs'),*Path('tests').rglob('*.rs'),*Path('benches').rglob('*.rs'),*Path('examples').rglob('*.rs'),*Path('tests/fixtures').rglob('*'),Path('Cargo.toml'),Path('Cargo.lock'),Path('tools/eval_ocr_metrics.py'),*Path('docs').glob('latin_dotted*.md')]:
  if p.is_file():add(str(p),p)
 inventory=json.loads((r/'product-rgb-inventory.json').read_text())
 for p in inventory:add(p,p)
 for p in [Path('tmp/latin-pair-policy/product-manifest.json'),Path('tmp/latin-pair-policy/fresh/evaluation-manifest.json')]:add(str(p),p)
 allowed={'.rs','.py','.toml','.lock','.md','.json','.jsonl','.stdout','.stderr','.log','.csv','.png','.jpg','.jpeg','.rgb','.txt','.swift','.yaml','.gz'}
 for directory,dirs,names in os.walk(r):
  dirs[:]=[d for d in dirs if d not in {'__pycache__','existing-replay','final-restored','restored-target','restored-source','packet-cache'}]
  for name in names:
   p=Path(directory)/name
   if p.suffix in allowed and name not in {'checkpoint.md','reproduce-command.json','reproduce.stdout','reproduce.stderr'}and not name.startswith('restored-'):add(str(p),p)
 for p in r.glob('comparison/*/existing-replay/records.json'):add(str(p),p)
 parent=Path('docs/benchmarks/latin-pair-bridge');refs={}
 for p in parent.glob('*.json'):add('parents/latin-pair-bridge/'+p.name,p)
 for p in parent.glob('*.zip'):refs[p.name]=sha(p.read_bytes())
 (out/'parents.json').write_text(json.dumps({'packet':'latin-pair-bridge','archives_sha256':refs,'verification_sha256':sha((parent/'verification.json').read_bytes()),'full_parent_recomputation_in_this_phase':False},indent=2)+'\n')
 seen={};archives={};part=0;members={};aliases={};z=None
 def start():return zipfile.ZipFile(out/('evidence-'+str(part)+'.zip'),'w',zipfile.ZIP_DEFLATED,compresslevel=6)
 def finish():
  nonlocal z,members,aliases
  name='evidence-'+str(part);key=name+'-aliases.json';b=(json.dumps(aliases,indent=2)+'\n').encode();z.writestr(key,b);members[key]=sha(b);z.close();p=out/(name+'.zip');size=p.stat().st_size;assert size<50*1024*1024;h=sha(p.read_bytes());(out/(name+'.json')).write_text(json.dumps({'sha256':h,'members':members},indent=2)+'\n');archives[name]=h;print(name,len(members),'members',len(aliases),'aliases',size,'bytes',flush=True);members={};aliases={}
 z=start()
 for key,p in sorted(files.items()):
  b=p.read_bytes();h=sha(b)
  if h in seen:aliases[key]={'source':seen[h],'sha256':h};continue
  estimate=len(zlib.compress(b,6))+len(key.encode())*2+200
  if z.fp.tell()+estimate>40*1024*1024 and members:finish();part+=1;z=start()
  seen[h]=key;z.writestr(key,b);members[key]=h
 finish();(out/'archives.json').write_text(json.dumps({'archives':archives,'files':len(files),'product_pairs':282465,'package_driver_sha256':sha(Path(__file__).read_bytes())},indent=2)+'\n');shutil.copy2(r/'reproduce.py',out/'reproduce.py')
if __name__=='__main__':main()
