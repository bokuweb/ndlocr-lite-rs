"""Re-score the frozen diagnostic, without running or selecting a model."""
from pathlib import Path
import gzip,hashlib,json,sys,zipfile
root=Path(__file__).resolve().parent
sys.path.insert(0,str(root.parents[2]/'tools'))
from eval_ocr_metrics import levenshtein
meta=json.loads((root/'probe-archive.json').read_text(encoding="utf-8"));packed=(root/'probe.json.gz').read_bytes()
assert hashlib.sha256(packed).hexdigest()==meta['gzip_sha256'];raw=gzip.decompress(packed);assert hashlib.sha256(raw).hexdigest()==meta['sha256']
rows=json.loads(raw)
with zipfile.ZipFile(root/'text-evidence.zip') as archive:
 truth={r['id']:archive.read('truth/'+r['id']+'.txt').decode('utf-8')for r in json.loads((root/'evaluation-manifest.json').read_text(encoding="utf-8"))['inputs']}
out=[]
for row in rows:
 if not row['variants']:continue
 before=row['variants'][0]['text'];expected=truth[row['id']]
 for variant in row['variants'][1:]:
  out.append({'id':row['id'],'mode':row['mode'],'index':row['index'],'padding':variant['padding'],'before_errors':levenshtein(list(expected),list(before)),'after_errors':levenshtein(list(expected),list(variant['text']))})
assert out==json.loads((root/'padding-audit.json').read_text(encoding="utf-8"))
for padding in [1,2,4,8]:
 group=[x for x in out if x['padding']==padding]
 print(padding,'crop comparisons',len(group),'regressions',sum(x['after_errors']>x['before_errors']for x in group),'improvements',sum(x['after_errors']<x['before_errors']for x in group))
