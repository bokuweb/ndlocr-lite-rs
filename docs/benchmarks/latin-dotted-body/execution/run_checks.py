from pathlib import Path
import json,subprocess,os,time,hashlib
r=Path('tmp/latin-dotted-body');sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest();parent=Path('tmp/latin-pair-policy');native=json.loads((parent/'native-tests-command.json').read_text());examples=json.loads((parent/'example-tests-command.json').read_text());env=native['env'];jobs=[('tests',['cargo','test','--locked']),('native-tests',native['argv']),('example-tests',examples['argv']),('clippy',['cargo','clippy','--locked','--features','onnx','--all-targets','--message-format=json'])];sources={str(p):sha(p)for p in [*Path('src').rglob('*.rs'),*Path('tests').rglob('*.rs'),*Path('examples').rglob('*.rs'),Path('Cargo.toml'),Path('Cargo.lock')]};(r/'check-freeze.json').write_text(json.dumps({'sources_sha256':sources,'driver_sha256':sha(__file__)},indent=2)+'\n')
for name,argv in jobs:
 rec={'argv':argv,'env':env};(r/(name+'-command.json')).write_text(json.dumps(rec,indent=2)+'\n');start=time.monotonic()
 with(r/(name+'.stdout')).open('wb')as a,(r/(name+'.stderr')).open('wb')as b:p=subprocess.run(argv,env=os.environ|env,stdout=a,stderr=b)
 rec.update(exit_code=p.returncode,seconds=time.monotonic()-start);(r/(name+'-command.json')).write_text(json.dumps(rec,indent=2)+'\n');assert p.returncode==0,name;assert all(sha(p)==h for p,h in sources.items());print(name,'success',round(rec['seconds'],2),flush=True)
