from pathlib import Path
import json,hashlib
r=Path('tmp/page-chunk-probe');f=json.loads((r/'results.json').read_text());n=json.loads((r/'native/results.json').read_text())
assert len(f['pages'])==len(n['pages'])==40
checks=[]
for p,q in zip(f['pages'],n['pages']):
 assert (p['image'],p['mode'])==(q['image'],q['mode'])
 assert len(p['lines'])==len(q['lines'])
 for a,b in zip(p['lines'],q['lines']):
  assert (a['index'],a['bounds'],a['before'])==(b['index'],b['bounds'],b['before'])
  a['raw_before']=b['raw_before']
  checks.append([p['image'],p['mode'],a['index'],a['bounds']])
f['native_groups']=False
(r/'results-enriched.json').write_text(json.dumps(f,ensure_ascii=False,indent=2)+'\n')
(r/'raw-baseline-provenance.json').write_text(json.dumps({'initial_sha256':hashlib.sha256((r/'results.json').read_bytes()).hexdigest(),'native_capture_sha256':hashlib.sha256((r/'native/results.json').read_bytes()).hexdigest(),'matched_baselines':len(checks),'details':checks},ensure_ascii=False,indent=2)+'\n')
print('raw baseline captures verified',len(checks))
