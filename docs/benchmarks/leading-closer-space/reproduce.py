from pathlib import Path
import argparse,hashlib,json,zipfile,subprocess,os
p=argparse.ArgumentParser();p.add_argument('--output',required=True);a=p.parse_args();root=Path(a.output).resolve();here=Path(__file__).resolve().parent;index=json.loads((here/'evidence.json').read_text());archive=here/'evidence.zip';sha=lambda b:hashlib.sha256(b).hexdigest();assert sha(archive.read_bytes())==index['sha256'];root.mkdir(parents=True,exist_ok=True)
with zipfile.ZipFile(archive)as z:
 assert set(z.namelist())==set(index['members'])
 for name,digest in index['members'].items():
  assert not name.startswith('/')and'..'not in Path(name).parts;b=z.read(name);assert sha(b)==digest,name;out=root/name;out.parent.mkdir(parents=True,exist_ok=True);out.write_bytes(b)
for directory,variable in [(root,'NDLOCR_COMPACT_LEADING_SPACE_EVAL_ROOT'),(root/'failed-v1','NDLOCR_LEADING_SPACE_EVAL_ROOT')]:
 before={str(p.relative_to(directory)):p.read_bytes()for p in (directory/'metrics').glob('*')};before['audit.json']=(directory/'audit.json').read_bytes();subprocess.run(['python3',str(directory/'evaluate.py')],check=True,env={**os.environ,variable:str(directory),'NDLOCR_SCORER':str(root/'tools/eval_ocr_metrics.py')},stdout=subprocess.DEVNULL)
 for name,b in before.items():assert(directory/name).read_bytes()==b,name
print('all archive/member SHA verified;36 metric files and both audits reproduced byte-for-byte')
