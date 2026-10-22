#!/usr/bin/env python3
"""Restore fixed narrow-round marker evidence without running ONNX."""
from pathlib import Path
import os,argparse,csv,hashlib,importlib.util,json,struct,sys,unicodedata,re

def sha(b):return hashlib.sha256(b).hexdigest()
def read(root,p):return json.loads((root/p).read_text())
def load_parents(packet,root):
 refs=read(packet,'parents.json');p=packet.parent/refs['packet'];script=p/'reproduce.py';assert sha(script.read_bytes())==refs['reproducer_sha256'];assert sha((p/'parents.json').read_bytes())==refs['nested_parents_json_sha256'];sys.dont_write_bytecode=True;spec=importlib.util.spec_from_file_location('verified_owned_parent',script);owned=importlib.util.module_from_spec(spec);spec.loader.exec_module(owned);internal,detached,bridge,ih,dh,bh=owned.load_parents(p,root/'parent');h=detached.restore(p,root/'parent',list(refs['archives']));assert h==refs['archives'];return owned,internal,detached,bridge,h,{'internal':ih,'detached':dh,'bridge':bh}

def verify_pixels(root,prior):
 from PIL import Image
 from portable_native import verify_native_trace
 import os
 data=root/'data/fresh';m=read(data,'evaluation-manifest.json');assert len(m['inputs'])==8064
 oldroot=Path('tmp/narrow-round-profile')
 def resolve(path):
  p=Path(path);assert '..'not in p.parts
  return root/'data'/p.relative_to(oldroot)
 work=root/'portable-pixel-work';work.mkdir();(work/'fresh').mkdir();(work/'fresh/native').symlink_to(data/'native',target_is_directory=True)
 for name in ['atlas.png','lines.json']:(work/'fresh'/name).symlink_to(data/name)
 (work/'manifest.json').write_text(json.dumps({'cases':prior},ensure_ascii=False)+'\n')
 port=m|{'inputs':[x|{k:str(resolve(x[k]))for k in ['original','image','rgb','effective_rgb','truth']}for x in m['inputs']]}
 (work/'fresh/evaluation-manifest.json').write_text(json.dumps(port,ensure_ascii=False)+'\n')
 for driver,audit in [('verify_fresh.py','fresh-pixel-audit.json'),('verify_noise.py','noise-pixel-audit.json')]:
  source=(root/'diagnostics'/driver).read_text();expected=read(root,'diagnostics/'+audit);assert sha(source.encode())==expected['driver_sha256']
  source=source.replace("r=Path('tmp/narrow-round-profile')",'r=Path('+repr(str(work))+')')
  source=source.replace("Path(run['font_file']).samefile(rec['argv'][1])","unicodedata.normalize('NFC',run['font_file'])==unicodedata.normalize('NFC',rec['argv'][1])")
  path=work/driver;path.write_text('import unicodedata\n'+source);exec(compile(path.read_text(),str(path),'exec'),{'__file__':str(path)})
  actual=read(work,audit)
  for k,v in actual.items():
   if k!='driver_sha256':assert expected[k]==v,(audit,k)
 assert len({(x['width'],x['height'],x['rgb_sha256'])for x in m['inputs']})==5568
 assert len({(x['width'],x['height'],x['rgb_sha256'])for x in m['inputs']if not x['seen_before_137991']})==5304
 for rec in m['native_records']:
  trace=read(data/'native',rec['id']+'.json');text=rec['call']['text']if rec['engine']=='freetype'else rec['argv'][4];verify_native_trace(rec,trace,text)
 captures=0
 for folder,n in [('fresh',8064),('horizontal',244)]:
  f=root/'data'/folder;manifest=read(f,'evaluation-manifest.json');rows=manifest['inputs'];assert len(rows)==n;xs={x['id']:x for x in rows};atlas=Image.open(f/'atlas.png').convert('RGB');assert sha((f/'atlas.png').read_bytes())==manifest['atlas_sha256']and sha((f/'lines.json').read_bytes())==manifest['lines_sha256']
  if folder=='horizontal':
   assert {x['id']for x in rows}=={x['id']for x in m['inputs']if x['rotated_for_auto']}
   for x in rows:
    for k in ['original','image','rgb','effective_rgb','truth']:assert sha(resolve(x[k]).read_bytes())==x[k+'_sha256']
    assert x['orientation']=='horizontal'and x['effective_rgb']==x['rgb'];assert atlas.crop(x['bounds']).tobytes()==resolve(x['rgb']).read_bytes()
  cap=read(f,'whole-rgb-capture/inputs.json');assert cap['model_input_dimensions']==[256,24]and len(cap['chunks'])==n;assert {c['line_id']for c in cap['chunks']}==set(xs)
  for c in cap['chunks']:
   x=xs[c['line_id']];im=Image.open(f/'whole-rgb-capture'/c['image']).convert('RGB');assert list(im.size)==c['dimensions']==[x['forward_width'],x['forward_height']];assert c['chunk_index']==0 and c['x_range']==[0,x['forward_width']];assert sha(im.tobytes())==x['effective_rgb_sha256'];captures+=1
 proof=read(data,'tensor-proof.json');assert proof=={'images':8064,'model_shapes':3,'all_original_auto_and_effective_horizontal_tensors_bit_equal':True}
 tensor=root/'tensor/fresh';tensor.mkdir(parents=True);(tensor/'evaluation-manifest.json').write_text(json.dumps(port,ensure_ascii=False)+'\n')
 return {'images':8308,'actual_captures':captures,'pre_jpeg_noise_pairs':2304,'native_records':648,'unique_fresh_rgb':5568,'unique_unseen_rgb':5304}

def reproduce_new_audit(root,bridge,cases,folder):
 corpus='narrow-'+folder;expected=read(root,'diagnostics/'+folder+'-policy/audit.json');data=read(root,'data/'+folder+'/evaluation-manifest.json');xs={x['id']:x for x in data['inputs']};groups={'all':list(xs),'seen':[k for k,x in xs.items()if x['seen_before_137991']],'unseen':[k for k,x in xs.items()if not x['seen_before_137991']],'text':[k for k,x in xs.items()if x['kind']=='text'],'noise':[k for k,x in xs.items()if x['kind']=='noise']};refs={(x['model'],x['id']):x for x in cases if x['corpus']==corpus};audit={k:expected[k]for k in ['scorer_sha256','driver_sha256']};audit|={'summary':[],'changes':[],'char_regressions':[],'word_regressions':[]};scorer=root/'diagnostics/source/eval_ocr_metrics.py';assert sha(scorer.read_bytes())==audit['scorer_sha256'];assert sha((root/'diagnostics/evaluate.py').read_bytes())==audit['driver_sha256'];tables=0
 for mode in sorted({x['engine']+'-p'+str(x['palt'])for x in xs.values()}):groups[mode]=[k for k,x in xs.items()if x['engine']+'-p'+str(x['palt'])==mode]
 for group,ids in groups.items():
  if not ids:continue
  truth=root/'generated/subsets'/folder/group/'truth';truth.mkdir(parents=True)
  for id in ids:os.link(root/'truth'/corpus/(id+'.txt'),truth/(id+'.txt'))
  for model in ['30','50','100']:
   base=root/'generated/subsets'/folder/group/model
   for phase,field in [('baseline','expected_before'),('candidate','expected_after')]:
    p=base/phase;p.mkdir(parents=True)
    for id in ids:os.link(root/'generated/core'/corpus/model/phase/(id+'.txt'),p/(id+'.txt'))
   for norm in ['none','width','strict']:
    rel='text/'+folder+'-policy/scores/'+group+'/'+model+'-'+norm+'.csv';target=root/'reproduced'/rel;bridge.score(scorer,truth,base/'baseline',base/'candidate',norm,target,root/rel,['baseline','candidate']);tables+=2;rows=list(csv.DictReader(target.open()));b={x['file']:x for x in rows if x['system']=='baseline'};a={x['file']:x for x in rows if x['system']=='candidate'};assert len(a)==len(b)==len(ids);stat={'group':group,'model':model,'normalization':norm,'images':len(ids),'changes':0,'char_improvements':0,'word_improvements':0,'exact_improvements':0}
    for id in b:
     bc,ac=int(b[id]['char_errors']),int(a[id]['char_errors']);bw,aw=int(b[id]['word_errors']),int(a[id]['word_errors']);changed=(base/'baseline'/id).read_bytes()!=(base/'candidate'/id).read_bytes();stat['changes']+=changed;stat['char_improvements']+=ac<bc;stat['word_improvements']+=aw<bw;stat['exact_improvements']+=bc>0 and ac==0
     if changed:audit['changes'].append([group,model,norm,id,bc,ac,bw,aw])
     if ac>bc:audit['char_regressions'].append([group,model,norm,id,bc,ac])
     if aw>bw:audit['word_regressions'].append([group,model,norm,id,bw,aw])
    audit['summary'].append(stat)
 assert audit==expected;return tables

def verify_source(root):
 current=root/'diagnostics/verification-source';prior=root/'parent/diagnostics/freetype/verification-source'
 for p in ['src/app.rs','src/infer/page_pool.rs','src/infer/parseq.rs','src/pipeline/run_page.rs','src/infer/visible_repetitions/single_pixel_bridge.rs']:assert(current/p).read_bytes()==(prior/p).read_bytes()
 block=b'\n#[cfg(test)]\n#[path = "../../tests/support/narrow_round_model.rs"]\nmod narrow_round_model_tests;\n';b=(current/'src/infer/cached.rs').read_bytes();assert block in b and b.replace(block,b'')==(prior/'src/infer/cached.rs').read_bytes()
 anchor=b'#[cfg(any(feature = "onnx", test))]\npub(crate) fn sanitize_visible_repetitions';a=(current/'src/infer/visible_repetitions.rs').read_bytes();b=(prior/'src/infer/visible_repetitions.rs').read_bytes();assert a[a.index(anchor):]==b[b.index(anchor):]
 path='src/infer/visible_repetitions/owned_parts_body_bridge.rs';a=(current/path).read_bytes();b=(prior/path).read_bytes();start=b'fn primary_support_mask';end=b'pub(super) fn supports';assert a[a.index(start):a.index(end)]==b[b.index(start):b.index(end)]
 for path in ['src/infer/visible_repetitions/narrow_round_marker.rs','src/infer/visible_repetitions/owned_parts_body_bridge.rs']:
  a=(root/'diagnostics/tensor-source'/path).read_text();b=(current/path).read_text();a=a[:a.index('#[cfg(test)]\nmod tests')];b=b[:b.index('#[cfg(test)]\nmod tests')]
  if path.endswith('/narrow_round_marker.rs'):a=a[a.index('pub(super) fn supports_mark'):];b=b[b.index('pub(super) fn supports_mark'):]
  norm=lambda s:re.sub(r'\s+','',s.replace('#[cfg(test)]',''));assert norm(a)==norm(b)
 for phase in ['probe','tensor','red','verification']:
  meta=read(root,'diagnostics/'+phase+'-command.json')
  for path,expected in meta['source_sha256'].items():assert sha((root/'diagnostics'/(phase+'-source')/path).read_bytes())==expected,(phase,path)
 checked=read(root,'diagnostics/checked-source-command.json');changed=[]
 for path,expected in checked['source_sha256'].items():
  after=(root/'diagnostics/checked-source'/path).read_bytes();assert sha(after)==expected;before=(current/path).read_bytes()
  if after!=before:
   assert path=='src/infer/visible_repetitions/narrow_round_marker.rs';assert after.count(b'rgb.chunks_mut(3)')==2;assert after.replace(b'rgb.chunks_mut(3)',b'rgb.chunks_exact_mut(3)')==before;changed.append(path)
 assert changed==checked['changed_test_source_files']and checked['all_runtime_rust_bytes_equal_measured_sources']

def main():
 from PIL import Image
 parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--output',type=Path,required=True);root=parser.parse_args().output.resolve();root.mkdir(parents=True);packet=Path(__file__).resolve().parent
 owned,internal,detached,bridge,parent_hashes,nested=load_parents(packet,root);hashes=detached.restore(packet,root,['images','pixels','inputs','diagnostics','text']);cases=read(root,'diagnostics/portable-cases.json')['cases'];assert len(cases)==162915;refs={(x['corpus'],x['model'],x['id']):x for x in cases};assert len(refs)==len(cases);prior=read(root,'parent/diagnostics/freetype/portable-cases.json')['cases'];assert len(prior)==137991
 for x in prior:
  y=refs[x['corpus'],x['model'],x['id']];assert(x['raw'],x['truth'],x['expected_after'],x['rgb_sha256'])==(y['raw'],y['truth'],y['expected_before'],y['rgb_sha256'])
 seen=set()
 for x in cases:
  if x['rgb']not in seen:assert sha((root/x['rgb']).read_bytes())==x['rgb_sha256'];seen.add(x['rgb'])
  p=root/'truth'/x['corpus']/(x['id']+'.txt');p.parent.mkdir(parents=True,exist_ok=True)
  if p.exists():assert p.read_bytes()==x['truth'].encode()
  else:p.write_bytes(x['truth'].encode())
  for phase,field in [('baseline','expected_before'),('candidate','expected_after')]:
   p=root/'generated/core'/x['corpus']/x['model']/phase/(x['id']+'.txt');p.parent.mkdir(parents=True,exist_ok=True);p.write_bytes(x[field].encode())
 corpora=['old','spaces','fresh','horizontal','new','positive','positive-horizontal','internal','internal-horizontal','owned-fresh','owned-horizontal'];tables=detached.reproduce_audit(bridge,root,cases,corpora,'text/scores','diagnostics/audit.json')
 for folder in ['fresh','horizontal']:tables+=reproduce_new_audit(root,bridge,cases,folder)
 assert tables==450
 means={};old=read(root,'parent/parent/parent/parent/diagnostics/parents/old-probe.json')
 for model in old['models']:
  for x in model['rows']:means['old',model['label'].removeprefix('single'),x['id']]=struct.unpack('<I',struct.pack('<f',x['mean_confidence']))[0]
 forwards=0
 for corpus,prefix,folder,n in [('owned-fresh','parent/diagnostics/freetype','fresh',7632),('owned-horizontal','parent/diagnostics/freetype','horizontal',24),('narrow-fresh','diagnostics','fresh',8064),('narrow-horizontal','diagnostics','horizontal',244)]:
  data=('parent/data/ft/'if corpus.startswith('owned-')else'data/')+folder;inputs={x['id']:x for x in read(root,data+'/evaluation-manifest.json')['inputs']}
  for model in ['30','50','100']:
   rows=read(root,prefix+'/'+folder+'/'+model+'-export.json');assert len(rows)==n and{x['id']for x in rows}==set(inputs);assert read(root,prefix+'/'+folder+'/'+model+'-command.json')['exit_code']==0
   for x in rows:
    key=corpus,model,x['id'];assert x['text']==refs[key]['raw']and x['bounds']==inputs[x['id']]['bounds'];assert len(x['chunks'])==1 and x['chunks'][0]['x_range']==[0,inputs[x['id']]['forward_width']]and not x['chunks'][0]['inserted_space_before'];means[key]=bridge.emitted_mean_bits(x);forwards+=corpus.startswith('narrow-')
 assert forwards==24924
 native=read(root,'diagnostics/native-manifest.json');pins={(x['corpus'],x['model'],x['id']):x for x in native['cases']};seen=set()
 for model in ['30','50','100']:
  rows=read(root,'diagnostics/native'+model+'-results.json');assert len(rows)==native['counts'][model];assert read(root,'diagnostics/native'+model+'-command.json')['exit_code']==0
  for x in rows:
   key=x['corpus'],model,x['id'];assert key not in seen and x['raw']==refs[key]['raw']and x['text']==refs[key]['expected_after'];assert x['mean_bits']==pins[key]['mean_bits']==means[key];seen.add(key)
 assert seen==set(pins)and len(pins)==8564
 cli=read(root,'diagnostics/line-cli-results.json');cm=read(root,'diagnostics/cli-manifest.json');inputs={(x['corpus'],x['model'],x['id']):x for x in read(root,'diagnostics/line-cli-command.json')['cases']};assert len(cli)==len(cm['cases'])==3500;expected_changed={(x['corpus'],x['model'],x['id'])for x in cases if x['selected']and x['orientation']=='auto'};assert {(x['corpus'],x['model'],x['id'])for x in cli if x['selected']}==expected_changed
 for x in cli:
  key=x['corpus'],x['model'],x['id'];assert x['exit_code']==0 and x['text']==refs[key]['expected_after'];name=x['model']+'-'+x['corpus']+'-'+x['id'];stdout=(root/'text/line-cli'/(name+'.stdout')).read_bytes();assert stdout[-1:]==b'\n'and stdout[:-1].decode()==x['text'];im=root/'text/line-cli/images'/(x['corpus']+'-'+x['id']+'.png');assert sha(im.read_bytes())==inputs[key]['image_sha256'];assert Image.open(im).convert('RGB').tobytes()==(root/refs[key]['rgb']).read_bytes()
 for table in(root/'text').rglob('*.csv'):
  rows=list(csv.DictReader(table.open()));systems={x['system']for x in rows};assert systems=={'baseline','candidate'};a={(x['file'],x['system']):x for x in rows}
  for file in{x['file']for x in rows}:
   b,c=a[file,'baseline'],a[file,'candidate'];assert int(c['char_errors'])<=int(b['char_errors'])and int(c['word_errors'])<=int(b['word_errors'])
 stats=verify_pixels(root,prior);verify_source(root);selected=[x for x in cases if x['selected']];assert len(selected)==62 and len({(x['width'],x['height'],x['rgb_sha256'])for x in selected})==19
 (root/'product-manifest.json').write_text(json.dumps({'cases':[x|{'rgb':str(root/x['rgb'])}for x in cases]},ensure_ascii=False,indent=2)+'\n');result={'archive_sha256':hashes,'parent_archives':parent_hashes,'nested_parent_archives':nested,'table_files_byte_equal':tables,'audits_recomputed_equal':3,'product_pairs':len(cases),'changes':len(selected),'unique_changed_original_rgb':19,'new_evaluation_raw_forwards':forwards,'native_api_pairs':len(pins),'cli_processes':len(cli),'page_fragment_source_proof_verified':True,'new_page_accuracy_evaluation_runs':0}|stats;(root/'verification.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(result,indent=2))
if __name__=='__main__':main()
