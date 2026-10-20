#!/usr/bin/env python3
"""Restore and independently rescore fixed Latin pair evidence without ONNX."""
from pathlib import Path
import argparse,csv,hashlib,importlib.util,json,os,struct,subprocess,sys,zipfile

def sha(b):return hashlib.sha256(b).hexdigest()
def read(root,path):return json.loads((root/path).read_text())
def restore(packet,root,names):
 result={}
 for name in names:
  index=read(packet,name+'.json');archive=packet/(name+'.zip');assert sha(archive.read_bytes())==index['sha256'];result[name]=index['sha256']
  with zipfile.ZipFile(archive)as z:
   assert set(z.namelist())==set(index['members'])
   for key,h in index['members'].items():
    p=Path(key);assert not p.is_absolute()and '..'not in p.parts
    data=z.read(key);assert sha(data)==h;target=root/p;target.parent.mkdir(parents=True,exist_ok=True)
    if target.exists():assert target.read_bytes()==data
    else:target.write_bytes(data)
   aliases=json.loads(z.read(name+'-aliases.json'))
   for key,a in aliases.items():
    p=Path(key);assert not p.is_absolute()and '..'not in p.parts
    assert a['source']in index['members']and a['sha256']==index['members'][a['source']]
    source=root/a['source'];assert sha(source.read_bytes())==a['sha256'];target=root/p;target.parent.mkdir(parents=True,exist_ok=True)
    if target.exists():assert target.read_bytes()==source.read_bytes()
    else:os.link(source,target)
 return result
def parent_full(packet,root):
 refs=read(packet,'parents.json');p=packet.parent/refs['packet'];script=p/'reproduce.py';assert sha(script.read_bytes())==refs['reproducer_sha256'];assert sha((p/'parents.json').read_bytes())==refs['nested_parents_json_sha256']
 for n,h in refs['archives'].items():assert read(p,n+'.json')['sha256']==h and sha((p/(n+'.zip')).read_bytes())==h
 sys.dont_write_bytecode=True;spec=importlib.util.spec_from_file_location('verified_narrow_parent',script);module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module);old=sys.argv
 try:sys.argv=[str(script),'--output',str(root/'parent')];module.main()
 finally:sys.argv=old
 actual=read(root/'parent','verification.json');assert actual['archive_sha256']==refs['archives']and actual['product_pairs']==162915
 return refs,actual
def score(root,truth,baseline,candidate,norm,relative):
 scorer=root/'diagnostics/source/eval_ocr_metrics.py';target=root/'reproduced'/relative;target.parent.mkdir(parents=True,exist_ok=True)
 subprocess.run([sys.executable,str(scorer),'--truth-dir',str(truth),'--system','baseline='+str(baseline),'--system','candidate='+str(candidate),'--normalize',norm,'--require-all','--output-csv',str(target),'--output-md',str(target.with_suffix('.md'))],check=True,capture_output=True)
 target.write_bytes(target.read_bytes().replace(b'\r\n',b'\n'));p=target.with_suffix('.md');p.write_bytes((p.read_text().rstrip()+'\n').encode())
 assert target.read_bytes()==(root/relative).read_bytes(),relative
 assert p.read_bytes()==(root/relative).with_suffix('.md').read_bytes(),str(p)
 return list(csv.DictReader(target.open()))
def audit_tables(root,cases,folder=None):
 expected=read(root,'diagnostics/'+(folder+'-policy/'if folder else'')+'audit.json');scorer=root/'diagnostics/source/eval_ocr_metrics.py';assert sha(scorer.read_bytes())==expected.get('scorer_sha256',read(root,'diagnostics/score-command.json')['scorer_sha256'])
 refs={(x['corpus'],x['model'],x['id']):x for x in cases};corpora=list(dict.fromkeys(x['corpus']for x in cases if x['corpus']not in ['latin-fresh','latin-horizontal']))
 if folder:
  corpus='latin-'+folder;xs={x['id']:x for x in read(root,'data/'+folder+'/evaluation-manifest.json')['inputs']}
  groups={'all':list(xs),'seen':[k for k,x in xs.items()if x['seen_before_162915']],'unseen':[k for k,x in xs.items()if not x['seen_before_162915']],'text':[k for k,x in xs.items()if x['kind']=='text'],'noise':[k for k,x in xs.items()if x['kind']=='noise']}
  for mode in sorted({x['engine']+'-p'+str(x['palt'])for x in xs.values()}):groups[mode]=[k for k,x in xs.items()if x['engine']+'-p'+str(x['palt'])==mode]
  for field in ['body','body_spaces','size','font']:
   for value in sorted({x[field]for x in xs.values()}):groups[field+'-'+str(value)]=[k for k,x in xs.items()if x[field]==value]
  for shape in sorted({x['shape']for x in xs.values()if x['kind']=='noise'}):groups['noise-'+shape]=[k for k,x in xs.items()if x.get('shape')==shape]
  jobs=[(group,corpus,ids)for group,ids in groups.items()if ids]
 else:jobs=[(c,c,[x['id']for x in cases if x['corpus']==c and x['model']=='30'])for c in corpora]
 audit={'summary':[],'changes':[],'char_regressions':[],'word_regressions':[]};tables=0
 for group,corpus,ids in jobs:
  truth=root/'generated/subsets'/('latin-'+folder if folder else'old')/group/'truth';truth.mkdir(parents=True)
  for id in ids:os.link(root/'truth'/corpus/(id+'.txt'),truth/(id+'.txt'))
  for model in ['30','50','100']:
   base=truth.parent/model
   for phase in ['baseline','candidate']:
    p=base/phase;p.mkdir(parents=True)
    for id in ids:os.link(root/'generated/core'/corpus/model/phase/(id+'.txt'),p/(id+'.txt'))
   for norm in ['none','width','strict']:
    rel='text/'+(folder+'-policy/scores/'+group if folder else'scores/'+corpus)+'/'+model+'-'+norm+'.csv'
    rows=score(root,truth,base/'baseline',base/'candidate',norm,rel);tables+=2;b={x['file']:x for x in rows if x['system']=='baseline'};a={x['file']:x for x in rows if x['system']=='candidate'};assert len(a)==len(b)==len(ids)
    stat={('group'if folder else'corpus'):group,'model':model,'normalization':norm,'images':len(ids),'changes':0,'char_improvements':0,'word_improvements':0,'exact_improvements':0}
    for file in b:
     bc,ac=int(b[file]['char_errors']),int(a[file]['char_errors']);bw,aw=int(b[file]['word_errors']),int(a[file]['word_errors']);x=refs[corpus,model,Path(file).stem];changed=x['expected_before']!=x['expected_after'];stat['changes']+=changed;stat['char_improvements']+=ac<bc;stat['word_improvements']+=aw<bw;stat['exact_improvements']+=bc>0 and ac==0
     if changed:audit['changes'].append([group,model,norm,file,bc,ac,bw,aw])
     if ac>bc:audit['char_regressions'].append([group,model,norm,file,bc,ac])
     if aw>bw:audit['word_regressions'].append([group,model,norm,file,bw,aw])
    audit['summary'].append(stat)
 for k in audit:
  if folder=='fresh'and k!='summary':assert sorted(audit[k])==sorted(expected[k]),k
  else:assert audit[k]==expected[k],(folder,k)
 assert not audit['char_regressions']and not audit['word_regressions']
 return tables

def verify_pixels(root,prior):
 from PIL import Image
 from portable_native import verify_native_trace
 import os
 data=root/'data/fresh';m=read(data,'evaluation-manifest.json');assert len(m['inputs'])==19440
 oldroot=Path('tmp/latin-pair-policy')
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
  source=source.replace("r=Path('tmp/latin-pair-policy')",'r=Path('+repr(str(work))+')')
  source=source.replace("Path(run['font_file']).samefile(rec['argv'][1])","unicodedata.normalize('NFC',run['font_file'])==unicodedata.normalize('NFC',rec['argv'][1])")
  path=work/driver;path.write_text('import unicodedata\n'+source);exec(compile(path.read_text(),str(path),'exec'),{'__file__':str(path)})
  actual=read(work,audit)
  for k,v in actual.items():
   if k!='driver_sha256':assert expected[k]==v,(audit,k)
 assert len({(x['width'],x['height'],x['rgb_sha256'])for x in m['inputs']})==13368
 assert len({(x['width'],x['height'],x['rgb_sha256'])for x in m['inputs']if not x['seen_before_162915']})==13368
 for rec in m['native_records']:
  trace=read(data/'native',rec['id']+'.json');text=rec['call']['text']if rec['engine']=='freetype'else rec['argv'][4];verify_native_trace(rec,trace,text)
 native_qa=read(root,'diagnostics/native-audit.json');assert native_qa['native_generation_records']==native_qa['independent_native_qa_redraws']==1854
 assert len(native_qa['records'])==1854
 for rec in native_qa['records']:
  im=Image.open(data/'native'/(rec['id']+'.png')).convert('RGB')
  assert rec['font_advance_glyph_and_rgb_verified'] and sha(im.tobytes())==rec['native_rgb_sha256']
 schema=(root/'diagnostics/test_fresh.py').read_text().replace("'tmp/latin-pair-policy/fresh/evaluation-manifest.json'",repr(str(work/'fresh/evaluation-manifest.json')))
 (work/'test_fresh.py').write_text(schema)
 subprocess.run([sys.executable,str(work/'test_fresh.py')],check=True,capture_output=True)
 captures=0
 for folder,n in [('fresh',19440),('horizontal',636)]:
  f=root/'data'/folder;manifest=read(f,'evaluation-manifest.json');rows=manifest['inputs'];assert len(rows)==n;xs={x['id']:x for x in rows};atlas=Image.open(f/'atlas.png').convert('RGB');assert sha((f/'atlas.png').read_bytes())==manifest['atlas_sha256']and sha((f/'lines.json').read_bytes())==manifest['lines_sha256']
  if folder=='horizontal':
   assert {x['id']for x in rows}=={x['id']for x in m['inputs']if x['rotated_for_auto']}
   for x in rows:
    for k in ['original','image','rgb','effective_rgb','truth']:assert sha(resolve(x[k]).read_bytes())==x[k+'_sha256']
    assert x['orientation']=='horizontal'and x['effective_rgb']==x['rgb'];assert atlas.crop(x['bounds']).tobytes()==resolve(x['rgb']).read_bytes()
  cap=read(f,'whole-rgb-capture/inputs.json');assert cap['model_input_dimensions']==[256,24]and len(cap['chunks'])==n;assert {c['line_id']for c in cap['chunks']}==set(xs)
  for c in cap['chunks']:
   x=xs[c['line_id']];im=Image.open(f/'whole-rgb-capture'/c['image']).convert('RGB');assert list(im.size)==c['dimensions']==[x['forward_width'],x['forward_height']];assert c['chunk_index']==0 and c['x_range']==[0,x['forward_width']];assert sha(im.tobytes())==x['effective_rgb_sha256'];captures+=1
 proof=read(data,'tensor-proof.json');assert proof=={'images':19440,'model_shapes':3,'all_original_auto_and_effective_horizontal_tensors_bit_equal':True}
 tensor=root/'tensor/fresh';tensor.mkdir(parents=True);(tensor/'evaluation-manifest.json').write_text(json.dumps(port,ensure_ascii=False)+'\n')
 return {'images':20076,'actual_captures':captures,'pre_jpeg_noise_pairs':4536,'native_records':1854,'unique_fresh_rgb':13368,'unique_unseen_rgb':13368}

def verify_source(root):
 current=root/'diagnostics/verification-source';prior=root/'parent/diagnostics/checked-source'
 a=(prior/'src/infer/visible_repetitions.rs').read_bytes();b=(current/'src/infer/visible_repetitions.rs').read_bytes()
 expected=a.replace(b'mod narrow_round_marker;',b'mod latin_pair_bridge;\nmod latin_pair_policy;\nmod narrow_round_marker;',1)
 anchor=b'        if owned_parts_body_bridge::supports_narrow_round(&fragment, rgb, width, height) {\n            return fragment;\n        }'
 expected=expected.replace(anchor,anchor+b'\n        if latin_pair_policy::supports(&fragment, rgb, width, height) {\n            return fragment;\n        }',1);assert b==expected
 marker=b'pub(crate) fn sanitize_visible_repetitions';assert a[a.index(marker):]==b[b.index(marker):]
 path='src/infer/cached.rs';assert(current/path).read_bytes()==(prior/path).read_bytes()+b'\n#[cfg(test)]\n#[path = "../../tests/support/latin_pair_model.rs"]\nmod latin_pair_model_tests;\n'
 path='src/infer/visible_repetitions/narrow_round_marker.rs';a=(prior/path).read_bytes();prefix,suffix=a.split(b'#[cfg(test)]',1);assert prefix.count(b'body_height')==5;assert(current/path).read_bytes()==prefix.replace(b'body_height',b'reference_height')+b'#[cfg(test)]'+suffix
 policy=(current/'src/infer/visible_repetitions/latin_pair_policy.rs').read_bytes().split(b'#[cfg(test)]')[0]
 assert policy.replace(b'//! Support for an owned pair',b'//! Test-only support for an owned pair',1).replace(b'pub(super) fn supports(',b'fn supports(',1)==(root/'diagnostics/frozen-policy-source.rs').read_bytes()
 kernel=(current/'src/infer/visible_repetitions/latin_pair_bridge.rs').read_bytes().split(b'#[cfg(test)]')[0]
 assert kernel.replace(b'//! Structural view of one raster neck',b'//! Test-only exploration of one raster neck',1).replace(b'pub(super) fn split_pair',b'fn split_pair')==(root/'diagnostics/kernel/frozen-kernel-source.rs').read_bytes()
 skip={'src/infer/visible_repetitions.rs','src/infer/cached.rs','src/infer/visible_repetitions/narrow_round_marker.rs'}
 checked=read(root,'parent/diagnostics/checked-source-command.json')
 for path,h in checked['source_sha256'].items():
  assert sha((prior/path).read_bytes())==h
  if path not in skip:assert(current/path).read_bytes()==(prior/path).read_bytes(),path
 for phase in ['probe','tensor','seed','verification']:
  meta=read(root,'diagnostics/'+phase+'-command.json')
  for path,h in meta['source_sha256'].items():assert sha((root/'diagnostics'/(phase+'-source')/path).read_bytes())==h,(phase,path)
 cli=read(root,'diagnostics/cli-binary.json')
 for path,h in cli['source_sha256'].items():assert sha((root/'diagnostics/cli-source'/path).read_bytes())==h
 before=read(root,'parent/diagnostics/verification-command.json')['models'];after=read(root,'diagnostics/verification-command.json')['models'];assert before==after==cli['models']
 raw=read(root,'diagnostics/fresh/raw-command.json')['raw_source_capture'];assert raw['models']==after
 assert sha((current/'Cargo.lock').read_bytes())==raw['source_sha256']['Cargo.lock']
 for path,h in raw['source_sha256'].items():assert sha((root/'diagnostics/fresh/export-source'/path).read_bytes())==h
 modelproof=read(root,'diagnostics/model-orchestration-source-proof.json');assert sha((root/'diagnostics/pre-nn-freeze/evaluate.py').read_bytes())==modelproof['frozen_driver_sha256'];assert sha((root/'diagnostics/evaluate_model.py').read_bytes())==modelproof['model_driver_sha256']
 combined=read(root,'diagnostics/fresh-policy/audit.json');assert combined['driver_sha256']==sha((root/'diagnostics/combine_model_evaluations.py').read_bytes())
 for model,files in combined['source_model_evaluations'].items():
  for path,h in files.items():assert sha((root/'diagnostics'/('fresh-policy-model'+model)/path).read_bytes())==h,(model,path)
 return True

def main():
 from PIL import Image
 parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--output',type=Path,required=True);args=parser.parse_args();root=args.output.resolve();root.mkdir(parents=True);packet=Path(__file__).resolve().parent
 parent_refs,parent_verified=parent_full(packet,root);index=read(packet,'archives.json');hashes=restore(packet,root,list(index['archives']));assert hashes==index['archives']
 cases=read(root,'diagnostics/portable-cases.json')['cases'];assert len(cases)==223143;refs={(x['corpus'],x['model'],x['id']):x for x in cases};assert len(refs)==223143;prior=read(root,'parent/diagnostics/portable-cases.json')['cases'];assert len(prior)==162915;parents={(x['corpus'],x['model'],x['id']):x for x in prior}
 seen=set()
 for x in cases:
  key=x['corpus'],x['model'],x['id'];original=root/x['rgb']
  if x['rgb']not in seen:assert sha(original.read_bytes())==x['rgb_sha256'];seen.add(x['rgb'])
  truth=root/'truth'/x['corpus']/(x['id']+'.txt');truth.parent.mkdir(parents=True,exist_ok=True)
  if not truth.exists():
   source=root/'parent/truth'/x['corpus']/(x['id']+'.txt')if key in parents else root/'data/fresh/truth'/(x['id']+'.txt');assert source.read_bytes()==x['truth'].encode();os.link(source,truth)
  else:assert truth.read_bytes()==x['truth'].encode()
  if key in parents:
   y=parents[key];assert(x['raw'],x['truth'],x['expected_before'],x['rgb_sha256'])==(y['raw'],y['truth'],y['expected_after'],y['rgb_sha256'])
  for phase,field in [('baseline','expected_before'),('candidate','expected_after')]:
   p=root/'generated/core'/x['corpus']/x['model']/phase/(x['id']+'.txt');p.parent.mkdir(parents=True,exist_ok=True)
   source=root/'parent/generated/core'/x['corpus']/x['model']/'candidate'/(x['id']+'.txt')
   if key in parents and x[field]==parents[key]['expected_after']:os.link(source,p)
   elif phase=='candidate'and x['expected_before']==x['expected_after']:os.link(p.parent.parent/'baseline'/p.name,p)
   else:p.write_bytes(x[field].encode())
 tables=audit_tables(root,cases)+audit_tables(root,cases,'fresh')+audit_tables(root,cases,'horizontal');assert tables==1188
 sys.path.insert(0,str(root/'diagnostics/source'));from means import emitted_means
 means={}
 # The inherited packet retains the original probe inside its verified ancestor.
 old=read(root,'parent/parent/parent/parent/parent/diagnostics/parents/old-probe.json')
 for m in old['models']:
  for x in m['rows']:means['old',m['label'].removeprefix('single'),x['id']]=struct.unpack('<I',struct.pack('<f',x['mean_confidence']))[0]
 forwards=0
 for folder,n in [('fresh',19440),('horizontal',636)]:
  xs={x['id']:x for x in read(root,'data/'+folder+'/evaluation-manifest.json')['inputs']}
  for model in ['30','50','100']:
   rows=read(root,'diagnostics/'+folder+'/'+model+'-export.json');assert len(rows)==n and{x['id']for x in rows}==set(xs);assert read(root,'diagnostics/'+folder+'/'+model+'-command.json')['exit_code']==0
   for x in rows:
    key='latin-'+folder,model,x['id'];c=x['chunks'];assert x['text']==refs[key]['raw']and x['bounds']==xs[x['id']]['bounds']and len(c)==1;assert c[0]['x_range']==[0,xs[x['id']]['forward_width']]and not c[0]['inserted_space_before'];means[key]=struct.unpack('<I',struct.pack('<f',emitted_means(x)[0]))[0];forwards+=1
 assert forwards==60228
 native=read(root,'diagnostics/native-manifest.json');pins={(x['corpus'],x['model'],x['id']):x for x in native['cases']};coverage=set()
 for model in ['30','50','100']:
  rows=read(root,'diagnostics/native'+model+'-results.json');assert len(rows)==native['counts'][model]and read(root,'diagnostics/native'+model+'-command.json')['exit_code']==0
  for x in rows:
   key=x['corpus'],model,x['id'];assert key not in coverage;assert x['raw']==refs[key]['raw']and x['text']==refs[key]['expected_after'];assert x['mean_bits']==pins[key]['mean_bits']==means[key];coverage.add(key)
 assert coverage==set(pins)and len(coverage)==9184
 controls=read(root,'diagnostics/fixed-noise-control-selection.json')['cases'];assert len(controls)==9072;labels=['30','50','100']
 for x in controls:assert x['model']==labels[hashlib.sha256(x['id'].encode()).digest()[0]%3]and x['orientation']=='auto'
 selected={key for key,x in refs.items()if x['selected']};expected=selected|{(x['corpus'],x['model'],x['id'])for x in controls};assert expected==coverage
 cli=read(root,'diagnostics/line-cli-results.json');cm=read(root,'diagnostics/line-cli-command.json');inputs={(x['corpus'],x['model'],x['id']):x for x in cm['cases']};assert len(cli)==9184 and{(x['corpus'],x['model'],x['id'])for x in cli}==expected
 for x in cli:
  key=x['corpus'],x['model'],x['id'];assert x['exit_code']==0 and x['text']==refs[key]['expected_after'];name=x['model']+'-'+x['corpus']+'-'+x['id'];stdout=(root/'text/line-cli'/(name+'.stdout')).read_bytes();assert stdout[-1:]==b'\n'and stdout[:-1].decode()==x['text'];image=root/'text/line-cli/images'/(x['corpus']+'-'+x['id']+'.png');assert sha(image.read_bytes())==inputs[key]['image_sha256'];assert Image.open(image).convert('RGB').tobytes()==(root/refs[key]['rgb']).read_bytes()
 stats=verify_pixels(root,prior);verify_source(root);assert len(selected)==112 and len({(refs[k]['width'],refs[k]['height'],refs[k]['rgb_sha256'])for k in selected})==48
 product={'cases':[x|{'rgb':str(root/x['rgb'])}for x in cases]};(root/'product-manifest.json').write_text(json.dumps(product,ensure_ascii=False,indent=2)+'\n')
 result={'archive_sha256':hashes,'parent_archive_sha256':parent_refs['archives'],'parent_fully_reproduced':True,'parent_verification_sha256':sha((root/'parent/verification.json').read_bytes()),'product_pairs':223143,'changes':112,'unique_changed_original_rgb':48,'table_files_byte_equal':tables,'audits_recomputed_equal':3,'new_evaluation_raw_forwards':forwards,'native_api_pairs':9184,'cli_processes':9184,'fixed_numeric_source_verified':True,'new_page_accuracy_evaluation_runs':0}|stats
 (root/'verification.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(result,indent=2))
if __name__=='__main__':main()
