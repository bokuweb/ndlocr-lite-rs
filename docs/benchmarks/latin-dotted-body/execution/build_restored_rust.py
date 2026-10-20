from pathlib import Path
import subprocess,json,hashlib,os,shutil,time
r=Path('tmp/latin-dotted-body');root=(r/'final-restored').resolve();sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest();expected=json.loads((r/'check-freeze.json').read_text())['sources_sha256']
for p,h in expected.items():assert sha(root/p)==h,p
argv=['cargo','test','--locked','--lib','--no-run','--message-format=json'];env={'CARGO_TARGET_DIR':str((r/'restored-target').resolve())};record={'argv':argv,'cwd':str(root),'env':env,'sources_sha256':expected,'driver_sha256':sha(__file__),'new_nn_forwards':0};start=time.monotonic()
with(r/'restored-build.jsonl').open('wb')as a,(r/'restored-build.stderr').open('wb')as b:d=subprocess.run(argv,cwd=root,env=os.environ|env,stdout=a,stderr=b)
record.update(exit_code=d.returncode,seconds=time.monotonic()-start);assert d.returncode==0
es=[json.loads(s)for s in(r/'restored-build.jsonl').read_text().splitlines()if s.startswith('{')];b=next(x['executable']for x in es if x.get('reason')=='compiler-artifact'and x.get('executable')and x['target']['name']=='ndlocr_lite_rs');shutil.copy2(b,r/'restored-rust-binary');record['binary_sha256']=sha(r/'restored-rust-binary')
for p,h in expected.items():assert sha(root/p)==h,p
(r/'restored-build-command.json').write_text(json.dumps(record,indent=2)+'\n');print('independently restored default source compiled; source and lock unchanged')
