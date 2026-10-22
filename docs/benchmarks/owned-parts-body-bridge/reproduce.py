#!/usr/bin/env python3
"""Restore fixed bridge/owned-parts evidence without running ONNX."""
from pathlib import Path
import argparse,csv,hashlib,importlib.util,json,struct,sys,unicodedata,re

def sha(b):return hashlib.sha256(b).hexdigest()
def read(root,p):return json.loads((root/p).read_text())
def load_parents(packet,root):
 refs=read(packet,'parents.json');p=packet.parent/refs['packet'];script=p/'reproduce.py';assert sha(script.read_bytes())==refs['reproducer_sha256'];assert sha((p/'parents.json').read_bytes())==refs['nested_parents_json_sha256'];sys.dont_write_bytecode=True;spec=importlib.util.spec_from_file_location('verified_internal_parent',script);internal=importlib.util.module_from_spec(spec);spec.loader.exec_module(internal);detached,bridge,hashes,nested=internal.load_parent(p,root/'parent');h=detached.restore(p,root/'parent',list(refs['archives']));assert h==refs['archives'];return internal,detached,bridge,h,hashes,nested

def verify_pixels(root,prior):
 from PIL import Image,ImageChops
 stats={'images':0,'actual_captures':0,'pre_jpeg_noise_pairs':0,'native_records':0,'geometry_only_images':0}
 oldkeys={(x['width'],x['height'],x['rgb_sha256'])for x in prior};matches={}
 for q in prior:
  key=q['width'],q['height'],q['rgb_sha256'];value={k:q.get(k)for k in ['corpus','id','orientation','truth']}
  if value not in matches.setdefault(key,[]):matches[key].append(value)
 scopes=[('ft/fresh','tmp/owned-parts-bridge/freetype/fresh',7632,True),('ft/horizontal','tmp/owned-parts-bridge/freetype/horizontal',24,True),('ct/fresh','tmp/owned-parts-bridge/fresh',10944,False),('native-screen/fresh','tmp/owned-parts-bridge/native-screen/fresh',16,False),('freetype-screen/fresh','tmp/owned-parts-bridge/freetype-screen/fresh',10,False)]
 def resolve(path):
  q=Path(path)
  for target,source,_,_ in sorted(scopes,key=lambda x:-len(x[1])):
   if q.is_relative_to(source):return root/'data'/target/q.relative_to(source)
  raise AssertionError(path)
 for scope,oldroot,count,measured in scopes[:3]:
  data=root/'data'/scope;m=read(data,'evaluation-manifest.json');rows=m['inputs'];assert len(rows)==count and len({x['id']for x in rows})==count;atlas=Image.open(data/'atlas.png').convert('RGB');assert sha((data/'atlas.png').read_bytes())==m['atlas_sha256'];assert sha((data/'lines.json').read_bytes())==m['lines_sha256']
  for x in rows:
   for k in ['rgb','effective_rgb','truth','image','original']:assert sha(resolve(x[k]).read_bytes())==x[k+'_sha256']
   im=Image.open(resolve(x['image'])).convert('RGB');assert im.tobytes()==resolve(x['rgb']).read_bytes();assert Image.open(resolve(x['original'])).convert('RGB').tobytes()==im.tobytes();eff=resolve(x['effective_rgb']).read_bytes();assert atlas.crop(x['bounds']).tobytes()==eff
   if scope=='ft/horizontal':assert x['orientation']=='horizontal'and im.tobytes()==eff
   else:
    rot=5*x['height']>4*x['width'];assert x['rotated_for_auto']==rot;assert(im.transpose(Image.Transpose.ROTATE_90)if rot else im).tobytes()==eff
   if scope.startswith('ft/'):
    assert x['layout_engine']=='BASIC'and x['palt']is None;assert x['seen_before_115023']==((x['width'],x['height'],x['rgb_sha256'])in oldkeys);assert x['prior_rgb_matches']==matches.get((x['width'],x['height'],x['rgb_sha256']),[])
   if scope in ['ft/fresh','ct/fresh']and x['kind']=='noise'and x['format']=='png':
    stem=f'd_t{x["body_id"]}_bs{x["body_spaces"]}_{x["font"]}_{x["size"]}_p{x["palt"]}_bare';full=Image.open(data/'native'/(stem+'.png')).convert('RGB');bb=ImageChops.difference(full,Image.new('RGB',full.size,'white')).getbbox();body=full.crop((bb[0]-2,bb[1]-2,bb[2]+2,bb[3]+2));sx,sy,ex,ey=x['shape_bounds'];shift=ex-sx+x['gap'];offset=shift if x['leading']else 0;expected=Image.new('RGB',(body.width+shift,body.height),'white');expected.paste(body,(offset,0));assert not any(body.getpixel((xx,yy))!=(255,255,255)and sx<=xx+offset<ex and sy<=yy<ey for yy in range(body.height)for xx in range(body.width))
    if ex>sx:expected.paste((0,0,0),(sx,sy,ex,ey))
    assert expected.tobytes()==im.tobytes();stats['pre_jpeg_noise_pairs']+=1
   stats['images']+=1;stats['geometry_only_images']+=not measured
  commands=m.get('native_records',m.get('native_commands',[]))
  for q in commands:
   native=data/'native'/(q['id']+'.png');tp=native.with_suffix('.json');assert sha(native.read_bytes())==q['png_sha256']and sha(tp.read_bytes())==q['trace_sha256'];t=json.loads(tp.read_text())
   if 'call'in q:
    c=q['call'];assert t['text']==c['text']and t['font']==c['font']and t['size']==c['size']and t['layout_engine']=='BASIC'and t['palt']is None and t['face_index']==0;assert [g['character']for g in t['glyph_masks']]==list(dict.fromkeys(t['text']));assert all(g['different_from_missing_glyph']for g in t['glyph_masks']);assert len(t['prefix_lengths'])==len(t['text'])+1
   else:
    norm=lambda p:unicodedata.normalize('NFC',p);assert norm(t['font_file'])==norm(q['argv'][1])
    for run in t['runs']:assert norm(run['font_file'])==norm(t['font_file'])and run['font']==t['font']and 0 not in run['glyphs']
    assert sorted(i for run in t['runs']for i in run['string_indices'])==list(range(len(q['argv'][4])))
   stats['native_records']+=1
  if measured:
   cap=read(data,'whole-rgb-capture/inputs.json');assert cap['model_input_dimensions']==[256,24]and len(cap['chunks'])==count;ids={x['id']:x for x in rows};assert{c['line_id']for c in cap['chunks']}==set(ids)
   for c in cap['chunks']:
    x=ids[c['line_id']];im=Image.open(data/'whole-rgb-capture'/c['image']).convert('RGB');assert list(im.size)==c['dimensions']==[x['forward_width'],x['forward_height']];assert c['chunk_index']==0 and c['x_range']==[0,x['forward_width']];assert sha(im.tobytes())==x['effective_rgb_sha256'];stats['actual_captures']+=1
  if scope=='ft/fresh':
   proof=read(data,'tensor-proof.json');assert proof=={'images':7632,'model_shapes':3,'all_original_auto_and_effective_horizontal_tensors_bit_equal':True};port=m|{'inputs':[x|{k:str(resolve(x[k]))for k in ['rgb','effective_rgb']}for x in rows]};h=root/'tensor/fresh';h.mkdir(parents=True);(h/'evaluation-manifest.json').write_text(json.dumps(port,ensure_ascii=False,indent=2)+'\n')
 for scope,oldroot,count,_ in scopes[3:]:
  data=root/'data'/scope;m=read(data,'evaluation-manifest.json');assert len(m['inputs'])==count
  for x in m['inputs']:
   for k in ['rgb','truth','image','original']:assert sha(resolve(x[k]).read_bytes())==x[k+'_sha256']
   im=Image.open(resolve(x['image'])).convert('RGB');assert list(im.size)==[x['width'],x['height']]and im.tobytes()==resolve(x['rgb']).read_bytes();assert resolve(x['truth']).read_text()==x['word']+'」'
  commands=m.get('native_records',m.get('native_commands',[]));assert len(commands)==count
  for q in commands:
   if 'argv'in q:
    png=resolve(q['argv'][-1]);tp=png.with_suffix('.json');t=json.loads(tp.read_text());norm=lambda p:unicodedata.normalize('NFC',p);assert norm(t['font_file'])==norm(q['argv'][1]);assert sorted(i for run in t['runs']for i in run['string_indices'])==list(range(len(q['argv'][4])))
    for run in t['runs']:assert norm(run['font_file'])==norm(t['font_file'])and run['font']==t['font']and 0 not in run['glyphs']
   else:
    png=data/(q['id']+'.png');tp=data/(q['id']+'-trace.json');t=json.loads(tp.read_text());assert t['layout_engine']=='BASIC'and t['palt']is None and all(g['different_from_missing_glyph']for g in t['glyph_masks'])
   assert sha(png.read_bytes())==q['png_sha256']and sha(tp.read_bytes())==q['trace_sha256']
  stats['images']+=count;stats['geometry_only_images']+=count;stats['native_records']+=count
 return stats

def reproduce_new_audit(root,bridge,cases,folder):
 corpus='owned-'+folder;expected=read(root,'diagnostics/freetype/'+folder+'-policy/audit.json');data=read(root,'data/ft/'+folder+'/evaluation-manifest.json');xs={x['id']:x for x in data['inputs']};groups={'all':list(xs),'seen':[k for k,x in xs.items()if x['seen_before_115023']],'unseen':[k for k,x in xs.items()if not x['seen_before_115023']],'text':[k for k,x in xs.items()if x['kind']=='text'],'noise':[k for k,x in xs.items()if x['kind']=='noise']};refs={(x['model'],x['id']):x for x in cases if x['corpus']==corpus};audit={k:expected[k]for k in ['scorer_sha256','driver_sha256']};audit|={'summary':[],'changes':[],'char_regressions':[],'word_regressions':[]};scorer=root/'diagnostics/source/eval_ocr_metrics.py';assert sha(scorer.read_bytes())==audit['scorer_sha256'];assert sha((root/'diagnostics/freetype/evaluate.py').read_bytes())==audit['driver_sha256'];tables=0
 for group,ids in groups.items():
  if not ids:continue
  truth=root/'generated/subsets'/folder/group/'truth';truth.mkdir(parents=True)
  for id in ids:(truth/(id+'.txt')).write_bytes(refs['30',id]['truth'].encode())
  for model in ['30','50','100']:
   base=root/'generated/subsets'/folder/group/model
   for phase,field in [('baseline','expected_before'),('candidate','expected_after')]:
    p=base/phase;p.mkdir(parents=True)
    for id in ids:(p/(id+'.txt')).write_bytes(refs[model,id][field].encode())
   for norm in ['none','width','strict']:
    rel='text/freetype/'+folder+'-policy/scores/'+group+'/'+model+'-'+norm+'.csv';target=root/'reproduced'/rel;bridge.score(scorer,truth,base/'baseline',base/'candidate',norm,target,root/rel,['baseline','candidate']);tables+=2;rows=list(csv.DictReader(target.open()));b={x['file']:x for x in rows if x['system']=='baseline'};a={x['file']:x for x in rows if x['system']=='candidate'};assert len(a)==len(b)==len(ids);stat={'group':group,'model':model,'normalization':norm,'images':len(ids),'changes':0,'char_improvements':0,'word_improvements':0,'exact_improvements':0}
    for id in b:
     bc,ac=int(b[id]['char_errors']),int(a[id]['char_errors']);bw,aw=int(b[id]['word_errors']),int(a[id]['word_errors']);changed=(base/'baseline'/id).read_bytes()!=(base/'candidate'/id).read_bytes();stat['changes']+=changed;stat['char_improvements']+=ac<bc;stat['word_improvements']+=aw<bw;stat['exact_improvements']+=bc>0 and ac==0
     if changed:audit['changes'].append([group,model,norm,id,bc,ac,bw,aw])
     if ac>bc:audit['char_regressions'].append([group,model,norm,id,bc,ac])
     if aw>bw:audit['word_regressions'].append([group,model,norm,id,bw,aw])
    audit['summary'].append(stat)
 assert audit==expected;return tables

def verify_source(root):
 current=root/'diagnostics/freetype/verification-source';prior=root/'parent/diagnostics/verification-source'
 for p in ['src/app.rs','src/infer/page_pool.rs','src/infer/parseq.rs','src/pipeline/run_page.rs']:assert(current/p).read_bytes()==(prior/p).read_bytes()
 block=b'\n#[cfg(test)]\n#[path = "../../tests/support/owned_parts_bridge_model.rs"]\nmod owned_parts_bridge_model_tests;\n';b=(current/'src/infer/cached.rs').read_bytes();assert block in b and b.replace(block,b'')==(prior/'src/infer/cached.rs').read_bytes();anchor=b'#[cfg(any(feature = "onnx", test))]\npub(crate) fn sanitize_visible_repetitions';a=(current/'src/infer/visible_repetitions.rs').read_bytes();b=(prior/'src/infer/visible_repetitions.rs').read_bytes();assert a[a.index(anchor):]==b[b.index(anchor):];a=(current/'src/infer/visible_repetitions/single_pixel_bridge.rs').read_bytes();assert a.replace(b'pub(super) fn split_body',b'fn split_body').replace(b'pub(super) fn take_region',b'fn take_region')==(prior/'src/infer/visible_repetitions/single_pixel_bridge.rs').read_bytes()
 path='src/infer/visible_repetitions/owned_parts_body_bridge.rs';a=(root/'diagnostics/probe-source'/path).read_text().split('#[cfg(test)]')[0];b=(current/path).read_text().split('#[cfg(test)]')[0];normalize=lambda s:re.sub(r'\s+','',s[s.index('fn primary_support_mask'):].replace('pub(super) fn supports','fn supports'));assert normalize(a)==normalize(b)

def main():
 from PIL import Image
 parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--output',type=Path,required=True);root=parser.parse_args().output.resolve();root.mkdir(parents=True);packet=Path(__file__).resolve().parent;internal,detached,bridge,h,nh,nested=load_parents(packet,root);names=['images','pixels','geometry-images','geometry-pixels','screens','inputs','diagnostics','text'];hashes=detached.restore(packet,root,names);cases=read(root,'diagnostics/freetype/portable-cases.json')['cases'];assert len(cases)==137991;refs={(x['corpus'],x['model'],x['id']):x for x in cases};assert len(refs)==len(cases);prior=read(root,'parent/diagnostics/portable-cases.json')['cases']
 for x in prior:
  y=refs[x['corpus'],x['model'],x['id']];assert(x['raw'],x['truth'],x['expected_after'],x['rgb_sha256'])==(y['raw'],y['truth'],y['expected_before'],y['rgb_sha256'])
 seen=set()
 for x in cases:
  if x['rgb']not in seen:assert sha((root/x['rgb']).read_bytes())==x['rgb_sha256'];seen.add(x['rgb'])
  p=root/'truth'/x['corpus']/(x['id']+'.txt');p.parent.mkdir(parents=True,exist_ok=True)
  if p.exists():assert p.read_bytes()==x['truth'].encode()
  p.write_bytes(x['truth'].encode())
  for phase,field in [('baseline','expected_before'),('candidate','expected_after')]:
   p=root/'generated/core'/x['corpus']/x['model']/phase/(x['id']+'.txt');p.parent.mkdir(parents=True,exist_ok=True);p.write_bytes(x[field].encode())
 corpora=['old','spaces','fresh','horizontal','new','positive','positive-horizontal','internal','internal-horizontal'];tables=detached.reproduce_audit(bridge,root,cases,corpora,'text/scores','diagnostics/audit.json')
 for folder in ['fresh','horizontal']:tables+=reproduce_new_audit(root,bridge,cases,folder)
 means={};old=read(root,'parent/parent/parent/diagnostics/parents/old-probe.json')
 for model in old['models']:
  for x in model['rows']:means['old',model['label'].removeprefix('single'),x['id']]=struct.unpack('<I',struct.pack('<f',x['mean_confidence']))[0]
 forwards=0
 for folder,n in [('fresh',7632),('horizontal',24)]:
  corpus='owned-'+folder;inputs={x['id']:x for x in read(root,'data/ft/'+folder+'/evaluation-manifest.json')['inputs']}
  for model in ['30','50','100']:
   rows=read(root,'diagnostics/freetype/'+folder+'/'+model+'-export.json');assert len(rows)==n and{x['id']for x in rows}==set(inputs);assert read(root,'diagnostics/freetype/'+folder+'/'+model+'-command.json')['exit_code']==0
   for x in rows:
    key=corpus,model,x['id'];assert x['text']==refs[key]['raw']and x['bounds']==inputs[x['id']]['bounds'];assert len(x['chunks'])==1 and x['chunks'][0]['x_range']==[0,inputs[x['id']]['forward_width']]and not x['chunks'][0]['inserted_space_before'];means[key]=bridge.emitted_mean_bits(x);forwards+=1
 assert forwards==22968
 native=read(root,'diagnostics/freetype/native-manifest.json');pins={(x['corpus'],x['model'],x['id']):x for x in native['cases']};seen=set()
 for model in ['30','50','100']:
  rows=read(root,'diagnostics/freetype/native'+model+'-results.json');assert len(rows)==native['counts'][model]
  for x in rows:
   key=x['corpus'],model,x['id'];assert key not in seen and x['raw']==refs[key]['raw']and x['text']==refs[key]['expected_after'];assert x['mean_bits']==pins[key]['mean_bits']==means[key];seen.add(key)
 assert seen==set(pins);cli=read(root,'diagnostics/freetype/line-cli-results.json');cm=read(root,'diagnostics/freetype/cli-manifest.json');inputs={(x['corpus'],x['model'],x['id']):x for x in read(root,'diagnostics/freetype/line-cli-command.json')['cases']};assert len(cli)==len(cm['cases']);assert sum(x['selected']for x in cli)==sum(x['selected']for x in cases)
 for x in cli:
  key=x['corpus'],x['model'],x['id'];assert x['exit_code']==0 and x['text']==refs[key]['expected_after'];name=x['model']+'-'+x['corpus']+'-'+x['id'];stdout=(root/'text/freetype/line-cli'/(name+'.stdout')).read_bytes();assert stdout[-1:]==b'\n'and stdout[:-1].decode()==x['text'];im=root/'text/freetype/line-cli/images'/(x['corpus']+'-'+x['id']+'.png');assert sha(im.read_bytes())==inputs[key]['image_sha256'];assert Image.open(im).convert('RGB').tobytes()==(root/refs[key]['rgb']).read_bytes()
 stats=verify_pixels(root,prior);assert stats=={'images':18626,'actual_captures':7656,'pre_jpeg_noise_pairs':1728,'native_records':3374,'geometry_only_images':10970};verify_source(root)
 portable=[x|{'rgb':str(root/x['rgb'])}for x in cases];(root/'product-manifest.json').write_text(json.dumps({'cases':portable},ensure_ascii=False,indent=2)+'\n');result={'archive_sha256':hashes,'parent_archives':h,'nested_parent_archives':nh,'further_parent_archives':nested,'table_files_byte_equal':tables,'audits_recomputed_equal':3,'product_pairs':len(cases),'new_evaluation_raw_forwards':forwards,'native_api_pairs':len(pins),'cli_processes':len(cli),'page_fragment_source_proof_verified':True,'new_page_nn_runs':0}|stats;(root/'verification.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(result,indent=2))
if __name__=='__main__':main()
