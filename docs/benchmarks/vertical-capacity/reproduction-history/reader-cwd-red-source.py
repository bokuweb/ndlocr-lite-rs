#!/usr/bin/env python3
"""Restore and independently verify capacity-recovery evidence without models."""
from pathlib import Path,PurePosixPath
import argparse,collections,csv,hashlib,json,os,random,shutil,struct,subprocess,sys,zipfile
from fractions import Fraction

def sha(data):return hashlib.sha256(data).hexdigest()
def read(path):return json.loads(Path(path).read_text(encoding='utf-8'))
def safe(key):
 p=PurePosixPath(key)
 if p.is_absolute() or '..' in p.parts or not p.parts or '\\' in key or ':' in key:raise ValueError('unsafe member path')
 return Path(*p.parts)
def restore(packet,output):
 packet,output=Path(packet),Path(output)
 if output.exists() and any(output.iterdir()):raise ValueError('output must be empty')
 indexes=read(packet/'archives.json')['archives']
 if set(indexes)!={'inputs','evidence'}:raise ValueError('full inputs and completed NN evidence archives required')
 for name,h in indexes.items():
  index=read(packet/(name+'.json'))
  if index['sha256']!=h or sha((packet/(name+'.zip')).read_bytes())!=h:raise ValueError('archive hash mismatch')
 output.mkdir(parents=True,exist_ok=True);restored={}
 for name in indexes:
  index=read(packet/(name+'.json'))
  with zipfile.ZipFile(packet/(name+'.zip'))as z:
   assert set(z.namelist())==set(index['members']) and len(z.namelist())==len(index['members'])
   for key,h in index['members'].items():
    target=output/safe(key);raw=z.read(key);assert sha(raw)==h
    if key in restored:assert restored[key]==h and target.read_bytes()==raw
    else:target.parent.mkdir(parents=True,exist_ok=True);target.write_bytes(raw);restored[key]=h
   aliases=read(output/index.get('alias_member',name+'-aliases.json'))
   for key,alias in aliases.items():
    target=output/safe(key);source=output/safe(alias['source']);assert alias['source']in restored and restored[alias['source']]==alias['sha256']==sha(source.read_bytes()) and key not in restored
    target.parent.mkdir(parents=True,exist_ok=True)
    try:os.link(source,target)
    except OSError:shutil.copyfile(source,target)
    restored[key]=alias['sha256']
 return {'archives':indexes,'restored_paths':len(restored)}

def verify_inputs(root):
 from PIL import Image
 r=root/'tmp/vertical-capacity';old=root/'tmp/real-vertical-scan';rows=read(r/'page-regression/manifest.json')['inputs'];assert len(rows)==293
 for x in rows:
  for k in ('image','truth'):assert sha((root/x[k]).read_bytes())==x[k+'_sha256']
 oldpages=read(old/'evaluation-manifest-before-nn.json')['inputs'];assert len(oldpages)==3
 for x in oldpages:
  for k in ('original','image','rgb','truth'):assert sha((root/x[k]).read_bytes())==x[k+'_sha256']
  actual=Image.open(root/x['image']).convert('RGB');assert actual.tobytes()==(root/x['rgb']).read_bytes()==Image.open(root/x['original']).convert('RGB').crop(x['crop']).tobytes();assert(root/x['truth']).read_bytes()=='\n'.join(x['text_columns']).encode()
 c=r/'controls';controls=read(c/'all-manifest.json')['inputs'];assert len(controls)==27;downloads={x['canvas_index']:x for x in read(c/'downloads.json')}
 for x in controls:
  for k in ('image','rgb','truth'):assert sha((root/x[k]).read_bytes())==x[k+'_sha256']
  im=Image.open(root/x['image']).convert('RGB');assert list(im.size)==[x['width'],x['height']] and im.tobytes()==(root/x['rgb']).read_bytes()
  if x['kind'].startswith('real_'):
   source=downloads[x['canvas']];assert sha((root/source['path']).read_bytes())==source['sha256'];assert im.tobytes()==Image.open(root/source['path']).convert('RGB').crop(x['box']).tobytes()
   if x['kind']=='real_body':assert(root/x['truth']).read_bytes()=='\n'.join(x['text_columns']).encode()
  else:
   n=x['count'];w,h=44*n+20,50;paper={'white':(255,255,255),'warm':(240,220,170),'gray':(225,225,225)}[x['paper']];raw=bytearray(bytes(paper)*w*h);seed=int.from_bytes(hashlib.sha256(f"vertical-capacity-noise-v1/{x['paper']}/{x['pattern']}/{n}".encode()).digest()[:8],'big');rng=random.Random(seed);points=[]
   for k in range(n):
    for y in range(32):
     for a in range(32):
      black=(a in range(7,25)) if x['pattern']=='bars' else (a-min(y,31-y) in (0,1,2)) if x['pattern']=='zigzag' else rng.random()<.5
      if black:
       pos=[10+44*k+a,9+y];points.append(pos);i=(pos[1]*w+pos[0])*3;raw[i:i+3]=bytes(3)
   assert points==read(root/x['points_path']);assert sha((root/x['points_path']).read_bytes())==x['points_sha256'];assert sha(raw)==x['designed_rgb_sha256'];assert im.transpose(Image.Transpose.ROTATE_90).tobytes()==raw
 newpages=read(c/'page-manifest.json')['inputs']
 for source,parents in [(old/'detected-columns/manifest.json',oldpages),(c/'detected-columns/manifest.json',newpages)]:
  columns=read(source)['inputs'];assert len(columns)==9
  for x in columns:
   p=next(p for p in parents if p['id']==x['parent_id']);im=Image.open(root/x['image']).convert('RGB');assert im.tobytes()==(root/x['rgb']).read_bytes()==Image.open(root/p['image']).convert('RGB').crop(x['bbox']).tobytes();assert(root/x['truth']).read_bytes()==p['text_columns'][x['column_index']].encode()
 v=read(r/'vertical-regression/manifest.json')['inputs'];assert len(v)==7;byid={x['id']:x for x in v}
 for x in read(root/'tests/fixtures/long_vertical/manifest.json')['items']:assert(root/byid['vertical_'+x['id']]['truth']).read_bytes()==x['truth'].encode()
 original=(root/'tests/fixtures/vertical_handwriting/truth.txt').read_bytes();assert original.endswith(b'\n') and (root/byid['vertical_handwriting']['truth']).read_bytes()==original[:-1]
 return {'page_inputs':293,'additional_known_vertical_inputs':7,'old_and_new_real_columns':18,'fixed_margin_figure_controls':24}

def otsu(gray):
 hist=collections.Counter(gray);n=len(gray);mass=sum(g*c for g,c in hist.items());best=None;chosen=None
 for t in range(255):
  low={g:c for g,c in hist.items()if g<=t};w=sum(low.values())
  if not 0<w<n:continue
  s=sum(g*c for g,c in low.items());score=Fraction((mass*w-s*n)**2,w*(n-w))
  if best is None or score>best:best=score;chosen=t
 return chosen

def verify_geometry(root):
 from PIL import Image
 r=root/'tmp/vertical-capacity';p=r/'policy';inputs={x['id']:x for x in read(p/'fragment-api-manifests/30.json')['inputs']};records=read(p/'all42-support-masks/results.json');assert len(records)==42
 for x in records:
  source=inputs[x['id']];rgb=(root/source['rgb']).read_bytes();rot=Image.frombytes('RGB',(source['width'],source['height']),rgb).transpose(Image.Transpose.ROTATE_90);actual=(p/'all42-support-masks'/x['original_file']).read_bytes();assert actual==rot.tobytes() and list(rot.size)==[x['width'],x['height']];gray=[sum(actual[i:i+3])//3 for i in range(0,len(actual),3)];t=otsu(gray);assert t==x['threshold']
  if t is None:assert x['support_file']is None
  else:assert(p/'all42-support-masks'/x['support_file']).read_bytes()==b''.join(bytes((0,0,0)if g<=t else(255,255,255))for g in gray)
 old={x['id']:x for x in read(root/'tmp/real-vertical-scan/detected-columns/manifest.json')['inputs']};geometric=0
 for folder in (r/'columns-prepared',r/'otsu/columns-prepared'):
  for x in read(folder/'records.json'):
   a=old[x['id']];rot=Image.frombytes('RGB',(a['width'],a['height']),(root/a['rgb']).read_bytes()).transpose(Image.Transpose.ROTATE_90);assert(folder/(x['id']+'-original.rgb')).read_bytes()==rot.tobytes()
   for model in x['models']:
    for chunk in model.get('chunks',[]):
     lo,hi=chunk['x_range'];assert 0<=lo<hi<=rot.width and not chunk['space_before'];assert(folder/chunk['file']).read_bytes()==rot.crop((lo,0,hi,rot.height)).tobytes();geometric+=1
 rows={x['id']:x for x in read(r/'controls/candidate-evaluation-manifest.json')['inputs']};callbacks=0;accepted=0
 for label in ('30','50','100'):
  for x in read(p/'candidate-results'/label/'results.json'):
   row=rows[x['id']];rot=Image.frombytes('RGB',(row['width'],row['height']),(root/row['rgb']).read_bytes()).transpose(Image.Transpose.ROTATE_90)
   for chunk in x['chunks']:
    lo,hi=chunk['x_range'];assert(chunk['width'],chunk['height'])==(hi-lo,rot.height);assert(p/'candidate-results'/label/chunk['file']).read_bytes()==rot.crop((lo,0,hi,rot.height)).tobytes();callbacks+=1
   if x['accepted']:
    parts=x['chunks'];text=''.join(c['text']for c in parts);assert text==x['selected']['text'];assert len(parts)>=2 and all(c['text'].strip()and .9<=c['mean_confidence']<=1 for c in parts);assert[c for c in text if c.isspace()]==[c for c in x['base']['text']if c.isspace()];assert len(set(c for c in text if c.isalnum()))>=4
    mean=sum(struct.unpack('<f',struct.pack('<I',c['mean_bits']))[0]*len(c['text'])for c in parts)/len(text);assert struct.unpack('<I',struct.pack('<f',mean))[0]==x['selected']['mean_bits'];accepted+=1
   else:assert x['selected']==x['base']
 assert geometric==153 and callbacks==81 and accepted==2
 return {'independent_fraction_masks':42,'h1_h2_geometrical_chunks':geometric,'actual_raw_chunk_coordinates':callbacks,'accepted_pairs':accepted}

def verify_sources_and_routes(root):
 r=root/'tmp/vertical-capacity/policy';product=read(r/'product-freeze.json');source=r/'product-source'
 for p,h in product['sources'].items():assert sha((source/p).read_bytes())==h
 for p,h in product['spec'].items():assert sha((r/'product-spec'/p).read_bytes())==h
 for filename,snapshot,decl in [('fragment-native-freeze.json','fragment-native-source','\n#[cfg(all(test, feature = "onnx"))]\nmod api_capture;\n'),('pure-replay-freeze.json','pure-replay-source','\n#[cfg(test)]\nmod replay;\n')]:
  meta=read(r/filename);folder=r/snapshot;hashes=meta.get('isolated_source',meta.get('source_sha256'))
  for p,h in hashes.items():assert sha((folder/p).read_bytes())==h
  for p in product['sources']:
   original=(source/p).read_bytes();actual=(folder/p).read_bytes();assert actual==(original+decl.encode()if p=='src/infer/vertical_capacity.rs'else original)
 old=root/'tmp/real-vertical-scan/initial-source'
 for p in [Path('src/infer/visible_repetitions.rs'),*[x.relative_to(source)for x in (source/'src/infer/visible_repetitions').rglob('*.rs')]]:assert(source/p).read_bytes()==(old/p).read_bytes()
 compared=0;negative=0
 kinds={x['id']:x['cohort']for x in read(root/'tmp/vertical-capacity/controls/candidate-evaluation-manifest.json')['inputs']}
 for label in ('30','50','100'):
  folder=r/'fragment-native-results'/label;assert read(folder/'failures.json')==[];comparisons=read(folder/'comparison.json');assert len(comparisons)==210
  for route in ('raw_session','normal_pool','fragment_single','fragment_batch','fragment_one_batch'):
   rows=read(folder/route/'results.json');assert len(rows)==42 and {x['id']for x in rows}==set(kinds)
   for x in rows:
    c=next(c for c in comparisons if c['route']==route and c['id']==x['id']);assert c['matches'] and x['text']==c['expected_text'] and x['mean_bits']==c['expected_mean_bits'];assert(folder/route/(x['id']+'.txt')).read_bytes()==x['text'].encode();compared+=1
    if route.startswith('fragment')and kinds[x['id']]in ('real_margin','artificial_stability'):negative+=1
 replay=read(r/'pure-replay-results/results.json');assert len(replay)==126 and all(x['matched']for x in replay)and sum(x['callbacks']for x in replay)==81 and sum(x['accepted']for x in replay)==2
 assert compared==630 and negative==216
 return {'native_api_pairs':compared,'negative_fragment_route_pairs':negative,'actual_rust_selector_replay_pairs':126,'stored_raw_callbacks':81}

def matrix_systems(root,directory):
 systems={};r=root/directory
 for phase in ('before','after'):
  for model in ('30','50','100'):
   for mode in ('eval','default'):systems[f'{phase}_{model}_{mode}']=r/phase/f'api-{model}'/f'runtime{model}_{mode}'
  for mode in ('deim','bands'):systems[f'{phase}_cli_{mode}']=r/phase/'cli'/mode
 return systems

def regenerate_tables(root):
 r=root/'tmp/vertical-capacity';old=root/'tmp/real-vertical-scan';groups=[]
 for directory in ('page-regression','vertical-regression'):groups.append((directory,r/directory/'scores',r/directory/'truth',matrix_systems(root,'tmp/vertical-capacity/'+directory),'before_cli_deim'))
 systems={f'{label}_{kind}':r/'policy/scores'/f'{label}_{kind}'for label in ('30','50','100')for kind in ('base','selected')};groups.append(('candidate',r/'policy/scores',r/'policy/scores/truth',systems,'30_base'))
 for key,parent,truth in [('controls-initial',r/'controls',r/'controls/body-truth'),('old-initial',old,old/'truth')]:
  systems={'cli_deim':parent/('baseline-pages/cli/deim'if key=='controls-initial'else'cli/deim'),'cli_bands':parent/('baseline-pages/cli/bands'if key=='controls-initial'else'cli/bands')}
  for model in ('30','50','100'):
   for mode in ('eval','default'):systems[f'runtime{model}_{mode}']=parent/('baseline-pages'if key=='controls-initial'else'')/f'api-{model}'/f'runtime{model}_{mode}'
  groups.append((key,parent/'initial-scores',truth,systems,'cli_deim'))
 systems={model+'_'+p.name:p for model in ('30','50','100')for p in sorted((old/('line-'+model)).iterdir())if p.is_dir()};groups.append(('old-line',old/'line-scores',old/'detected-columns/truth',systems,'30_raw_session'))
 for directory in ('partial-api-first','full-api'):
  models=read(r/'page-regression'/directory/'audit.json')['only_completed_models'];systems={}
  for model in models:
   for mode in ('eval','default'):
    for phase in ('before','after'):systems[f'{phase}_{model}_{mode}']=r/'page-regression'/phase/f'api-{model}'/f'runtime{model}_{mode}'
  groups.append((directory,r/'page-regression'/directory,r/'page-regression/truth',systems,'before_30_eval'))
 tables=0;rows_checked=0
 for name,expected,truth,systems,baseline in groups:
  out=root/'reproduced/tables'/name;out.mkdir(parents=True,exist_ok=True)
  for norm in ('none','width','strict'):
   table=out/(norm+'.csv');argv=[sys.executable,str(root/'tools/eval_ocr_metrics.py'),'--truth-dir',str(truth)]
   for label,folder in systems.items():argv+=['--system',label+'='+str(folder)]
   argv+=['--normalize',norm,'--require-all','--baseline-system',baseline,'--output-csv',str(table),'--output-md',str(table.with_suffix('.md'))];done=subprocess.run(argv,capture_output=True);(out/(norm+'.stdout')).write_bytes(done.stdout);(out/(norm+'.stderr')).write_bytes(done.stderr);assert done.returncode==0,(name,norm,done.stderr.decode());assert table.read_bytes()==(expected/(norm+'.csv')).read_bytes(),(name,norm,'CSV');assert table.with_suffix('.md').read_bytes()==(expected/(norm+'.md')).read_bytes(),(name,norm,'Markdown');tables+=2
   with table.open(newline='',encoding='utf-8')as stream:rows_checked+=sum(1 for _ in csv.DictReader(stream))
 assert tables==48 and rows_checked==33528
 # Recompute both char/word distances with an independent bit-vector algorithm.
 copies={}
 for name in ('page-regression','vertical-regression'):
  target=root/'reproduced/independent'/name;shutil.copytree(r/name,target);copies[name]=target
 for script,args in [('verify_metrics.py',[str(copies['page-regression'])]),('verify_metrics.py',[str(copies['vertical-regression'])]),('verify_full_api_metrics.py',[str(copies['page-regression'])])]:
  done=subprocess.run([sys.executable,str(r/'page-regression'/script),*args],cwd=root,capture_output=True);assert done.returncode==0,done.stderr.decode()
 return {'csv_markdown_files_recreated_byte_equal':tables,'formal_csv_rows_recreated':rows_checked,'independent_main_csv_rows':14064,'independent_vertical_csv_rows':336,'independent_full_api_csv_rows':10548}

def verify_matrix_boundaries(root):
 r=root/'tmp/vertical-capacity';total=0
 for directory,count in [('page-regression',293),('vertical-regression',7)]:
  folder=r/directory;manifest=read(folder/'manifest.json')['inputs'];receipt=read(folder/'after.json');audit=read(folder/'score-audit.json');assert len(manifest)==count and receipt['exit_code']==0 and receipt['all_frozen_materials_unchanged'] and not audit['regressions'];assert audit['comparisons']==count*8*3 and audit['all_same_line_counts']and audit['all_same_bboxes']
  for phase in ('before','after'):
   for model in ('30','50','100'):
    for mode in ('eval','default'):
     d=folder/phase/f'api-{model}'/f'runtime{model}_{mode}';rows=read(d/'results.json');assert len(rows)==count and {x['id']for x in rows}=={x['id']for x in manifest}
     for x in rows:assert x['text']=='\n'.join(l['text']for l in x['lines'])and(d/(x['id']+'.txt')).read_bytes()==x['text'].encode()
   for mode in ('deim','bands'):
    for x in manifest:
     id=x['id'];name=f'cli-{mode}-{id}';assert read(folder/phase/(name+'-command.json'))['exit_code']==0;assert(folder/phase/(name+'.stdout')).read_bytes()==(folder/phase/'cli'/mode/(id+'.txt')).read_bytes()+b'\n';total+=1
 assert total==1200
 return {'matrix_input_normalization_comparisons':7200,'cli_process_boundaries_verified':total}

def replay_rust(root):
 r=root/'tmp/vertical-capacity/policy';source=r/'pure-replay-source';fixtures=source/'tests/fixtures';shutil.copytree(root/'tests/fixtures',fixtures,dirs_exist_ok=True);target=root/'reproduced/rust-target';output=root/'reproduced/rust-selector';env=os.environ|{'CARGO_PROFILE_DEV_DEBUG':'0','CARGO_PROFILE_TEST_DEBUG':'0','CARGO_BUILD_JOBS':'2','CARGO_TARGET_DIR':str(target),'NDLOCR_VERTICAL_REPLAY_MANIFEST':str(r/'pure-replay-manifest.json'),'NDLOCR_VERTICAL_REPLAY_OUTPUT':str(output)};argv=['cargo','test','--locked','--manifest-path',str(source/'Cargo.toml'),'--lib','infer::vertical_capacity::replay::replay_fixed_candidates_without_nn','--','--exact','--ignored','--nocapture'];done=subprocess.run(argv,cwd=root,env=env,capture_output=True);(root/'reproduced/rust.stdout').write_bytes(done.stdout);(root/'reproduced/rust.stderr').write_bytes(done.stderr);assert done.returncode==0,done.stderr.decode();assert(output/'results.json').read_bytes()==(r/'pure-replay-results/results.json').read_bytes();return {'actual_rust_replayed_pairs':126,'callbacks_using_saved_raw_parts':81,'models_loaded':0,'new_nn_calls':0}

def main():
 parser=argparse.ArgumentParser();parser.add_argument('--output',type=Path,required=True);parser.add_argument('--rust',action='store_true',help='also rebuild the unchanged selector with the default backend; no ONNX/models');args=parser.parse_args();packet=Path(__file__).resolve().parent;root=args.output.resolve();result=restore(packet,root);result.update(verify_inputs(root));result.update(verify_geometry(root));result.update(verify_sources_and_routes(root));result.update(verify_matrix_boundaries(root));result.update(regenerate_tables(root));result.update(new_nn_calls=0,models_required=False,internal_nn_forward_count_in_recorded_page_calls=None)
 if args.rust:result.update(replay_rust(root))
 (root/'verification.json').write_text(json.dumps(result,indent=2)+'\n',encoding='utf-8');print(json.dumps(result,indent=2))
if __name__=='__main__':main()
