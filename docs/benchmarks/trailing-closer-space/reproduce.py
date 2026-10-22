from pathlib import Path
import argparse,hashlib,json,zipfile,subprocess,os
p=argparse.ArgumentParser();p.add_argument('--output',required=True);a=p.parse_args();root=Path(a.output).resolve();here=Path(__file__).resolve().parent;index=json.loads((here/'evidence.json').read_text());archive=here/'evidence.zip';sha=lambda b:hashlib.sha256(b).hexdigest();assert sha(archive.read_bytes())==index['sha256'];root.mkdir(parents=True,exist_ok=True)
with zipfile.ZipFile(archive)as z:
 assert set(z.namelist())==set(index['members'])
 for name,digest in index['members'].items():
  assert not name.startswith('/')and'..'not in Path(name).parts;b=z.read(name);assert sha(b)==digest,name;out=root/name;out.parent.mkdir(parents=True,exist_ok=True);out.write_bytes(b)
before={str(p.relative_to(root)):p.read_bytes()for p in (root/'metrics').glob('*')};before['audit.json']=(root/'audit.json').read_bytes();subprocess.run(['python3',str(root/'evaluate.py')],check=True,env={**os.environ,'NDLOCR_TRAILING_SPACE_EVAL_ROOT':str(root),'NDLOCR_SCORER':str(root/'tools/eval_ocr_metrics.py')},stdout=subprocess.DEVNULL)
for name,b in before.items():assert(root/name).read_bytes()==b,name
print('all archive/member SHA verified;18 metric files and audit reproduced byte-for-byte')
