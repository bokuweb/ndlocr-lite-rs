#!/usr/bin/env python3
"""Restore and independently rescore dotted Latin evidence without ONNX."""
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

def rescore(root):
 r=root/'tmp/latin-dotted-body';cases=read(r/'product-manifest.json')['cases'];assert len(cases)==282465;groups=collections.defaultdict(list)
 for x in cases:
  rgb=root/x['rgb'];assert sha(rgb.read_bytes())==x['rgb_sha256']and rgb.stat().st_size==3*x['width']*x['height'];groups[x['corpus'],x['model']].append(x)
 tables={};count=0
 for(corpus,model),xs in sorted(groups.items()):
  if corpus.startswith('dotted-'):expected=r/'comparison'/(corpus.removeprefix('dotted-')+'-'+model)/'scores';prefix=''
  else:expected=r/'scores'/corpus;prefix=model+'-'
  out=root/'reproduced-scores'/corpus/model;out.mkdir(parents=True)
  with tempfile.TemporaryDirectory(dir=root,prefix='score-')as d:
   d=Path(d)
   for part in ['truth','baseline','candidate']:(d/part).mkdir()
   for x in xs:
    safe(x['id']);assert Path(x['id']).name==x['id'];texts={'truth':x['truth'],'baseline':x.get('before',x['expected_after']),'candidate':x['expected_after']}
    for part,text in texts.items():(d/part/(x['id']+'.txt')).write_bytes(text.encode())
   for norm in ['none','width','strict']:
    table=out/(norm+'.csv');argv=[sys.executable,str(root/'tools/eval_ocr_metrics.py'),'--truth-dir',str(d/'truth'),'--system','baseline='+str(d/'baseline'),'--system','candidate='+str(d/'candidate'),'--normalize',norm,'--require-all','--baseline-system','baseline','--output-csv',str(table),'--output-md',str(table.with_suffix('.md'))];subprocess.run(argv,check=True,capture_output=True);assert table.read_bytes()==(expected/(prefix+norm+'.csv')).read_bytes(),(corpus,model,norm,'csv');assert table.with_suffix('.md').read_bytes()==(expected/(prefix+norm+'.md')).read_bytes(),(corpus,model,norm,'md');rows=list(csv.DictReader(table.open()));a={x['file']:x for x in rows if x['system']=='baseline'};b={x['file']:x for x in rows if x['system']=='candidate'};assert set(a)==set(b)=={x['id']+'.txt'for x in xs}
    for k in a:
     for metric in ['char','word']:assert int(b[k][metric+'_errors'])<=int(a[k][metric+'_errors'])
    tables[corpus,model,norm]=(a,b);count+=2
  print('rescored',corpus,model,len(xs),flush=True)
 assert count==306
 # Recompute every saved metadata aggregate from independently rescored per-input rows.
 cmeta={x['id']:x for x in read(root/'tmp/latin-pair-policy/fresh/evaluation-manifest.json')['inputs']};allcases={(x['corpus'],x['model'],x['id']):x for x in cases}
 aggregate=read(r/'aggregate-score-audit.json')
 for s in aggregate['summary']:
  if s['phase']=='saved':
   corpus=s['corpus'];ids=[x['id']for x in groups[corpus,s['model']]]
   if s['subset']!='all':ids=[i for i in ids if cmeta[i]['kind']==s['subset']]
  else:
   corpus='dotted-'+s['phase'];ids=read(r/'comparison'/(s['phase']+'-'+s['model'])/'score-groups.json')[s['group']]
  a,b=tables[corpus,s['model'],s['normalization']];assert len(ids)==s['pairs'];assert sum(allcases[corpus,s['model'],i].get('before',allcases[corpus,s['model'],i]['expected_after'])!=allcases[corpus,s['model'],i]['expected_after']for i in ids)==s['changes']
  for metric in ['char','word']:
   before=[int(a[i+'.txt'][metric+'_errors'])for i in ids];after=[int(b[i+'.txt'][metric+'_errors'])for i in ids];actual={'before':sum(before),'after':sum(after),'improved':sum(y<x for x,y in zip(before,after)),'regressed':sum(y>x for x,y in zip(before,after)),'equal':sum(y==x for x,y in zip(before,after)),'exact_recovered':sum(x>0 and y==0 for x,y in zip(before,after)),'exact_lost':sum(x==0 and y>0 for x,y in zip(before,after))};assert actual==s[metric],(s,metric,actual)
 return cases,count,len(aggregate['summary'])

def raw_and_apis(root,cases):
 r=root/'tmp/latin-dotted-body';refs={(x['corpus'],x['model'],x['id']):x for x in cases};sys.path.insert(0,str(r));from assemble_raw import assemble
 means={};forwards=0
 for phase,n in [('fresh',19440),('horizontal',334)]:
  m=read(r/phase/'evaluation-manifest.json')
  for model in ['30','50','100']:
   rows=read(r/phase/(model+'-export.json'));actual=assemble(m['inputs'],rows,model,phase);assert len(actual)==n and read(r/phase/(model+'-command.json'))['exit_code']==0
   for x in actual:
    k=x['corpus'],model,x['id'];assert all(x[f]==refs[k][f]for f in ['raw','truth','rgb_sha256','orientation','raw_mean_bits']);means[k]=x['raw_mean_bits'];forwards+=1
 assert forwards==59322
 probe=read(r/'old-probe.json')
 for m in probe['models']:
  for x in m['rows']:means['old',m['label'].removeprefix('single'),x['id']]=struct.unpack('<I',struct.pack('<f',x['mean_confidence']))[0]
 pins={(x['corpus'],x['model'],x['id']):x for x in read(r/'api-manifest.json')['cases']};covered=set()
 for model in ['30','50','100']:
  rows=read(r/('api-'+model+'-results.json'));assert read(r/('api-'+model+'-command.json'))['exit_code']==0
  for x in rows:
   k=x['corpus'],model,x['id'];assert k not in covered;assert x['raw']==refs[k]['raw']and x['text']==refs[k]['expected_after'];assert x['mean_bits']==pins[k]['mean_bits']==means[k];covered.add(k)
 assert covered==set(pins)and len(covered)==426
 fixed=read(r/'fixed-api-noise-selection.json')['inputs'];from api_selection import select_noise_api
 fresh=read(r/'fresh/evaluation-manifest.json')['inputs'];assert select_noise_api(fresh)==fixed
 selected={(x['corpus'],x['model'],x['id'])for x in cases if x.get('before',x['expected_after'])!=x['expected_after']};assert len(selected)==102;assert covered==selected|{('dotted-fresh',x['model'],x['id'])for x in fixed}
 cli=read(r/'cli-results.json');cpins={(x['corpus'],x['model'],x['id']):x for x in read(r/'cli-manifest.json')['cases']};assert len(cli)==len(cpins)==426;cli_expected={(('dotted-fresh'if c=='dotted-horizontal'else c),m,i)for c,m,i in covered};assert set(cpins)==cli_expected
 from PIL import Image
 for x in cli:
  k=x['corpus'],x['model'],x['id'];assert x['exit_code']==0 and x['text']==refs[k]['expected_after']and cpins[k]['orientation']=='auto';name=x['model']+'-'+x['corpus']+'-'+x['id'];b=(r/'line-cli'/(name+'.stdout')).read_bytes();assert b[-1:]==b'\n'and b[:-1].decode()==x['text'];image=r/'line-cli/images'/(x['corpus']+'-'+x['id']+'.png');assert Image.open(image).convert('RGB').tobytes()==(root/refs[k]['rgb']).read_bytes()
 return forwards,len(covered),len(cli)

def components(rgb,w,h):
 pixels={i for i in range(w*h)if sum(rgb[i*3:i*3+3])<660};parts=[]
 while pixels:
  pending=[pixels.pop()];found=[]
  while pending:
   i=pending.pop();found.append(i)
   for y in range(max(0,i//w-1),min(h,i//w+2)):
    for x in range(max(0,i%w-1),min(w,i%w+2)):
     j=y*w+x
     if j in pixels:pixels.remove(j);pending.append(j)
  parts.append([min(i%w for i in found),min(i//w for i in found),max(i%w for i in found)+1,max(i//w for i in found)+1,len(found)])
 return sorted(parts)

def pixels(root):
 from PIL import Image,ImageChops
 import runpy
 r=root/'tmp/latin-dotted-body';fresh=read(r/'fresh/evaluation-manifest.json');atlas=Image.open(r/'fresh/atlas.png').convert('RGB');prior={(x['width'],x['height'],x['rgb_sha256'])for x in read(root/'tmp/latin-pair-policy/product-manifest.json')['cases']};seen=set();rotated=0
 for x in fresh['inputs']:
  for field in ['original','image','rgb','effective_rgb','truth']:assert sha((root/x[field]).read_bytes())==x[field+'_sha256']
  im=Image.open(root/x['image']).convert('RGB');assert list(im.size)==[x['width'],x['height']]and im.tobytes()==(root/x['rgb']).read_bytes();assert Image.open(root/x['original']).convert('RGB').tobytes()==im.tobytes();auto=5*im.height>4*im.width;assert auto==x['rotated_for_auto'];effective=im.transpose(Image.Transpose.ROTATE_90)if auto else im;assert list(effective.size)==[x['forward_width'],x['forward_height']]and effective.tobytes()==(root/x['effective_rgb']).read_bytes()==atlas.crop(x['bounds']).tobytes();key=x['width'],x['height'],x['rgb_sha256'];assert key not in prior and not x['seen_before_223143'];seen.add(key);rotated+=auto
 assert len(fresh['inputs'])==19440 and len(seen)==13320 and rotated==334
 audit=read(r/'native-audit.json');assert len(audit['records'])==1854
 for rec in fresh['native_records']:
  p=r/'fresh/native'/(rec['id']+'.png');trace=p.with_suffix('.json');assert sha(p.read_bytes())==rec['png_sha256']and sha(trace.read_bytes())==rec['trace_sha256'];im=Image.open(p).convert('RGB');bb=ImageChops.difference(im,Image.new('RGB',im.size,'white')).getbbox();assert list(bb)==rec['native_ink_bbox'];crop=Image.new('RGB',(bb[2]-bb[0]+4,bb[3]-bb[1]+4),'white');crop.paste(im.crop(bb),(2,2));assert list(crop.size)==rec['cropped_dimensions']and sha(crop.tobytes())==rec['cropped_rgb_sha256'];rerender=Image.open(r/'native-rerender'/(rec['id']+'.png')).convert('RGB');assert rerender.tobytes()==im.tobytes()
 heights=read(r/'bare-component-audit/verification.json')['heights'];bare=read(r/'bare-component-audit/component-input.json')['rows'];assert len(bare)==126
 for x in bare:
  rgb=(root/x['rgb']).read_bytes();parts=components(rgb,x['width'],x['height']);assert parts==x['parts'];assert max(p[3]-p[1]for p in parts)==heights[x['native_id']]
 for helper,result in [('verify_capture.py','capture-audit.json'),('verify_noise.py','noise-pixel-audit.json')]:
  expected=(r/result).read_bytes();runpy.run_path(str(r/helper),run_name='__main__');assert(r/result).read_bytes()==expected,(helper,result)
 return {'unseen_conditions':19440,'unique_unseen_rgb':13320,'actual_nn_rgb_captures':19774,'saved_independent_native_redraws_verified':1854,'noise_png_reconstructed':4536}

def main():
 parser=argparse.ArgumentParser();parser.add_argument('--output',type=Path,required=True);args=parser.parse_args();packet=Path(__file__).resolve().parent;root=args.output.resolve();archives=restore(packet,root);old=Path.cwd();os.chdir(root)
 try:
  cases,tables,groups=rescore(root);forwards,api,cli=raw_and_apis(root,cases);stats=pixels(root)
  r=root/'tmp/latin-dotted-body';owner=(root/'src/infer/visible_repetitions/dotted_body.rs').read_text();owner=owner[owner.index('fn owner('):owner.index('\nfn collect_all(')].rstrip()+'\n';assert sha(owner.encode())==read(r/'kernel-lock.json')['owner_sha256'];policy=(root/'src/infer/visible_repetitions/dotted_body/policy.rs').read_text().split('\n#[cfg(test)]')[0].rstrip()+'\n';assert policy.split('\n',1)[1]==(r/'frozen-policy.rs').read_text().split('\n',1)[1]
  baseline=r/'policy-source/src'
  for old_source in baseline.rglob('*.rs'):
   relative=old_source.relative_to(baseline);name=str(relative)
   if name=='infer/visible_repetitions/dotted_body.rs' or name.startswith('infer/visible_repetitions/dotted_body/'):continue
   now=(root/'src'/relative).read_text();before=old_source.read_text()
   if name=='infer/visible_repetitions.rs':
    now=now.replace('mod dotted_body;','#[cfg(test)]\nmod dotted_body;').replace('        if dotted_body::supports(&fragment, rgb, width, height) {\n            return fragment;\n        }\n','')
   if name=='infer/cached.rs':now=now.removesuffix('\n#[cfg(test)]\n#[path = "../../tests/support/latin_dotted_model.rs"]\nmod latin_dotted_model_tests;\n')
   assert now==before,('product-source',name)
  product={'cases':[x|{'rgb':str(root/x['rgb'])}for x in cases]};(root/'product-manifest.json').write_text(json.dumps(product,ensure_ascii=False,indent=2)+'\n');verification={'archive_sha256':archives,'product_pairs':282465,'table_files_byte_equal':tables,'metadata_summaries_recomputed_equal':groups,'new_raw_forwards_verified':forwards,'api_pairs_verified':api,'cli_processes_verified':cli,'frozen_owner_and_policy_body_equal':True,'new_page_accuracy_runs':0,'parent_all_historical_tables_rerun':False}|stats;(root/'verification.json').write_text(json.dumps(verification,indent=2)+'\n');print(json.dumps(verification,indent=2))
 finally:os.chdir(old)
if __name__=='__main__':main()
