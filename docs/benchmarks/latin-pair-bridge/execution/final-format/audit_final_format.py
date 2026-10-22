from pathlib import Path
import json,hashlib,subprocess,shutil
r=Path('tmp/latin-pair-policy');out=Path('docs/benchmarks/latin-pair-bridge');sha=lambda b:hashlib.sha256(b).hexdigest();m=json.loads((r/'verification-command.json').read_text());changes={}
for name,h in m['source_sha256'].items():
 before=(r/'verification-source'/name).read_bytes();after=Path(name).read_bytes();assert sha(before)==h
 if before==after:continue
 assert name in ['src/infer/visible_repetitions/latin_pair_policy.rs','tests/support/latin_pair_model.rs'],name
 formatted=subprocess.run(['rustfmt','--edition','2024','--emit','stdout',str(r/'before-final-format'/name)],capture_output=True,check=True).stdout
 header=(str((r/'before-final-format'/name).resolve())+':\n\n').encode();assert formatted.startswith(header);formatted=formatted[len(header):]
 assert formatted==after,name
 if name.startswith('src/'):assert before.split(b'#[cfg(test)]')[0]==after.split(b'#[cfg(test)]')[0]
 changes[name]={'measured_sha256':sha(before),'final_sha256':sha(after),'exact_rustfmt_output':True,'runtime_prefix_unchanged':True}
assert len(changes)==2
dest=out/'execution/final-format';dest.mkdir(parents=True,exist_ok=True)
for name in changes:
 for phase,base in [('before',r/'verification-source'),('after',Path('.'))]:
  p=dest/phase/name;p.parent.mkdir(parents=True,exist_ok=True);shutil.copy2(base/name,p)
record={'changes':changes,'all_other_measured_source_bytes_equal':True,'all_runtime_rust_bytes_equal_measured':True,'numeric_conditions_unchanged':True,'reason':'CI cargo fmt check; only cfg(test) code formatted','initial_ci_run':39373776001,'initial_ci_head':'9b6070271bd56c56b359fa98328801f65b91ed5a','driver_sha256':sha(Path(__file__).read_bytes())}
(r/'final-format-audit.json').write_text(json.dumps(record,indent=2)+'\n');(dest/'audit.json').write_text(json.dumps(record,indent=2)+'\n');print('2 test-only files are exact rustfmt output; all runtime bytes unchanged',flush=True)
