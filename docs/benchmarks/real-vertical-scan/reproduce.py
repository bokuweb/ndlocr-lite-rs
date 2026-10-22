#!/usr/bin/env python3
"""Restore and independently rescore real vertical scan evidence without ONNX."""
from pathlib import Path
import argparse,hashlib,json,zipfile,os,sys,tempfile,subprocess,csv,collections,struct

def sha(b):return hashlib.sha256(b).hexdigest()
def read(path):return json.loads(Path(path).read_text())
def safe(key):
 p=Path(key)
 if p.is_absolute()or '..'in p.parts:raise ValueError('unsafe member path')
 return p

def restore(packet,output):
 packet,output=Path(packet),Path(output)
 if output.exists()and any(output.iterdir()):raise ValueError('output must be empty')
 indexes=read(packet/'archives.json')['archives']
 for name,h in indexes.items():
  safe(name);index=read(packet/(name+'.json'));assert index['sha256']==h==sha((packet/(name+'.zip')).read_bytes())
 output.mkdir(parents=True,exist_ok=True);restored={}
 for name,h in indexes.items():
  index=read(packet/(name+'.json'))
  with zipfile.ZipFile(packet/(name+'.zip'))as z:
   assert set(z.namelist())==set(index['members'])and len(z.namelist())==len(index['members'])
   for key,digest in index['members'].items():
    target=output/safe(key);data=z.read(key);assert sha(data)==digest
    if key in restored:assert restored[key]==digest and target.read_bytes()==data
    else:target.parent.mkdir(parents=True,exist_ok=True);target.write_bytes(data);restored[key]=digest
   aliases=json.loads(z.read(name+'-aliases.json'))
   for key,a in aliases.items():
    target=output/safe(key);source=output/safe(a['source']);assert a['source']in restored and restored[a['source']]==a['sha256']==sha(source.read_bytes());assert key not in restored
    target.parent.mkdir(parents=True,exist_ok=True);os.link(source,target);restored[key]=a['sha256']
 return indexes

def main():
 from PIL import Image
 import math
 parser=argparse.ArgumentParser();parser.add_argument('--output',type=Path,required=True);args=parser.parse_args();packet=Path(__file__).resolve().parent;root=args.output.resolve();hashes=restore(packet,root);r=root/'tmp/real-vertical-scan';manifest=read(r/'evaluation-manifest-before-nn.json');inputs=manifest['inputs'];assert len(inputs)==3 and manifest['truth_total_characters']==392
 m=read(r/'botchan-1884801-manifest.json');assert any(x['label']=='Access Restrictions'and x['value']=='PDM'for x in m['metadata']);manual=read(r/'manual-transcription.json')
 for x in inputs:
  for key in ['original','image','rgb','truth']:assert sha((root/x[key]).read_bytes())==x[key+'_sha256']
  im=Image.open(root/x['image']).convert('RGB');assert list(im.size)==[x['width'],x['height']]and im.tobytes()==(root/x['rgb']).read_bytes()==Image.open(root/x['original']).convert('RGB').crop(x['crop']).tobytes();assert(root/x['truth']).read_bytes()=='\n'.join(manual[str(x['canvas_index'])]).encode()
 systems={'cli_deim':r/'cli/deim','cli_bands':r/'cli/bands'}
 for model in ['30','50','100']:
  for mode in ['eval','default']:
   path=r/('api-'+model)/('runtime'+model+'_'+mode);systems['runtime'+model+'_'+mode]=path;rows=read(path/'results.json');assert len(rows)==3 and {x['id']for x in rows}=={x['id']for x in inputs}
   for x in rows:assert x['text']=='\n'.join(l['text']for l in x['lines'])and(path/(x['id']+'.txt')).read_bytes()==x['text'].encode()
 for mode in ['deim','bands']:
  for x in inputs:
   name='cli-'+mode+'-'+x['id'];assert(r/(name+'.stdout')).read_bytes()==(r/'cli'/mode/(x['id']+'.txt')).read_bytes()+b'\n';assert read(r/(name+'-command.json'))['exit_code']==0
 columns=read(r/'detected-columns/manifest.json')['inputs'];assert len(columns)==9
 for x in columns:
  p=next(p for p in inputs if p['id']==x['parent_id']);im=Image.open(root/x['image']).convert('RGB');assert im.tobytes()==(root/x['rgb']).read_bytes()==Image.open(root/p['image']).convert('RGB').crop(x['bbox']).tobytes();assert(root/x['truth']).read_bytes()==p['text_columns'][x['column_index']].encode()
 line_systems={};parity=[];bits=lambda f:struct.unpack('<I',struct.pack('<f',f))[0]
 for model in ['30','50','100']:
  pages={x['id']:x for x in read(r/('api-'+model)/('runtime'+model+'_eval')/'results.json')}
  raw={x['id']:x for x in read(r/('line-'+model)/'raw_session/results.json')}
  for path in sorted((r/('line-'+model)).iterdir()):
   line_systems[model+'_'+path.name]=path;rows=read(path/'results.json');assert len(rows)==9
   for x in rows:assert(path/(x['id']+'.txt')).read_bytes()==x['text'].encode()and math.isfinite(x['mean_confidence'])
  for x in columns:
   a=raw[x['id']];b=pages[x['parent_id']]['lines'][x['column_index']];parity.append({'model':model,'id':x['id'],'raw':a['text'],'page':b['text'],'raw_mean_bits':bits(a['mean_confidence']),'page_mean_bits':bits(b['confidence']),'text_equal':a['text']==b['text'],'mean_bits_equal':bits(a['mean_confidence'])==bits(b['confidence'])})
 assert parity==read(r/'line-score-audit.json')['parity'];assert sum(x['text_equal']for x in parity)==25 and all(x['mean_bits_equal']for x in parity)
 table_count=0
 for name,truth,paths,baseline,audit in [('initial-scores',r/'truth',systems,'cli_deim',read(r/'initial-score-audit.json')),('line-scores',r/'detected-columns/truth',line_systems,'30_raw_session',read(r/'line-score-audit.json'))]:
  out=root/'reproduced'/name;out.mkdir(parents=True);summaries=[]
  for norm in ['none','width','strict']:
   table=out/(norm+'.csv');argv=[sys.executable,str(root/'tools/eval_ocr_metrics.py'),'--truth-dir',str(truth)]
   for label,path in paths.items():argv+=['--system',label+'='+str(path)]
   argv+=['--normalize',norm,'--require-all','--baseline-system',baseline,'--output-csv',str(table),'--output-md',str(table.with_suffix('.md'))];subprocess.run(argv,check=True,capture_output=True);assert table.read_bytes()==(r/name/(norm+'.csv')).read_bytes();assert table.with_suffix('.md').read_bytes()==(r/name/(norm+'.md')).read_bytes();table_count+=2;rows=list(csv.DictReader(table.open()))
   for label in paths:
    xs=[x for x in rows if x['system']==label];s={'normalization':norm,'system':label,'char_errors':sum(int(x['char_errors'])for x in xs),'word_errors':sum(int(x['word_errors'])for x in xs)}
    if name=='initial-scores':s.update(char_total=sum(int(x['char_total'])for x in xs),word_total=sum(int(x['word_total'])for x in xs))
    else:s['pairs']=9
    summaries.append(s)
  assert summaries==audit['summary'],name
 initial=read(r/'initial-command.json');lines=read(r/'line-command.json');assert initial['exit_code']==lines['exit_code']==0;assert initial['all_source_models_inputs_unchanged_after']and lines['all_source_models_inputs_unchanged_after'];assert sha((root/'docs/real_vertical_scan_audit.md').read_bytes())!=initial['final_spec_sha256'];assert sha((r/'initial-spec.md').read_bytes())==initial['final_spec_sha256'];assert sha((r/'line-spec.md').read_bytes())==lines['spec_sha256']
 result={'archive_sha256':hashes,'baseline_commit':manifest['baseline_commit'],'original_canvases':3,'page_excerpt_inputs':3,'gt_characters_including_column_lf':392,'page_text_pairs_verified':24,'line_api_pairs_verified':216,'raw_page_pairs_verified':27,'raw_page_text_byte_equal':25,'raw_page_confidence_bit_equal':27,'table_files_byte_equal':table_count,'new_product_changes':0,'unprocessed_raw_calls_verified':27,'api_page_calls_verified':18,'cli_processes_verified':6,'model_internal_forward_count':None,'full_spread_accuracy_measured':False};(root/'verification.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(result,indent=2))
if __name__=='__main__':main()
