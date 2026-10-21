from pathlib import Path
import hashlib,io,json,sys,zipfile
from PIL import Image
sys.path.insert(0,'tools')
from vertical_page_matrix import Bundle,sha
from vertical_cli_fallback_full_matrix import merge_controls,full_plan
repo=Path.cwd();root=repo/'tmp/vertical-cli-fallback';pub=repo/'docs/benchmarks/vertical-cli-fallback/full-matrix-v1';pub.mkdir();base=Bundle(repo/'docs/benchmarks/vertical-page-fallback');groups=[{'origin':'base','inputs':base.index['physical_inputs'],'scoring_inputs':base.index['scoring_inputs']}];contents={};origins=[]
for origin,name in [('q','vertical-unique-insertion'),('s','vertical-low-stop-recovery'),('fvc','vertical-cli-fallback'),('real','real-vertical-additional-controls'),('c1','vertical-cli-model-agreement')]:
 path=repo/'tmp'/name/('additional-controls-manifest-v1.json' if origin=='real' else 'new-manifest.json');raw=path.read_bytes();master=json.loads(raw);rows=master['inputs'];scores=master['scoring_inputs'] if origin=='real' else [{'id':r['id'],'physical_id':r['id'],'truth':r['truth'],'truth_sha256':r['truth_sha256']} for r in rows];images=[];scoring=[];origins.append({'origin':origin,'manifest_path':str(path),'manifest_sha256':sha(raw),'physical_inputs':len(rows),'scoring_inputs':len(scores)})
 for row in rows:
  original=Path(row['image']);image=original if original.is_absolute() else repo/original;data=image.read_bytes();assert sha(data)==row['image_sha256'];im=Image.open(io.BytesIO(data)).convert('RGB');assert im.size==(row['width'],row['height']) and sha(im.tobytes())==row['rgb_sha256'];member='images/'+row['id']+'.png';assert member not in contents;contents[member]=data;images.append(dict(row,original_image_path=row['image'],image=member))
 for row in scores:
  original=Path(row['truth']);truth=original if original.is_absolute() else repo/original;data=truth.read_bytes();assert sha(data)==row['truth_sha256'];member='truth/'+row['id']+'.txt';assert member not in contents;contents[member]=data;scoring.append(dict(row,original_truth_path=row['truth'],truth=member))
 groups.append({'origin':origin,'inputs':images,'scoring_inputs':scoring})
physical,scoring=merge_controls(groups);assert len(physical)==1239 and len(scoring)==1245;assert len({(r['width'],r['height'],r['rgb_sha256']) for r in physical})==1238
for row in base.index['physical_inputs']:
 raw=base.read('data.zip',row['image']);im=Image.open(io.BytesIO(raw)).convert('RGB');assert im.size==(row['width'],row['height']) and sha(im.tobytes())==row['rgb_sha256']
archive=pub/'extra-data-v1.zip'
with zipfile.ZipFile(archive,'x',zipfile.ZIP_DEFLATED,compresslevel=9) as z:
 for n,data in sorted(contents.items()):z.writestr(n,data)
frozen={v:json.loads((root/f'cli-regression-{v}-frozen.json').read_text()) for v in ['baseline','candidate']};plan=full_plan(physical,scoring,8)
index={'schema':1,'archive':archive.name,'bytes':archive.stat().st_size,'sha256':sha(archive.read_bytes()),'members':{n:{'bytes':len(d),'sha256':sha(d)} for n,d in sorted(contents.items())},'physical_inputs':physical,'scoring_inputs':scoring,'source_files':{v:d['source_files'] for v,d in frozen.items()},'base_bundle_sha256':sha((base.folder/'bundle.json').read_bytes()),'adapter_packet_sha256':sha((pub.parent/'adapter-before-nn-v1.zip').read_bytes()),'new72_result_packet_sha256':sha((pub.parent/'new72-cli-matrix-v1.zip').read_bytes()),'assets':base.index['assets'],'expected':{'physical_inputs':1239,'scoring_inputs':1245,'unique_RGB':1238,'image_results':24780,'CSV_rows':74700},'shards':8,'source_manifests':origins,'producer_sha256':sha(Path(__file__).read_bytes()),'protocol_sha256':sha((repo/'docs/vertical_cli_fallback_full_gate.md').read_bytes()),'full_matrix_NN_calls_at_creation':0,'C1_policy_included':False}
(pub/'bundle.json').write_text(json.dumps(index,ensure_ascii=False,indent=2)+'\n');(pub/'plan.json').write_text(json.dumps(plan,ensure_ascii=False,indent=2)+'\n');(pub/'create_full_bundle_v1.py').write_bytes(Path(__file__).read_bytes());print(json.dumps({'extra_data_bytes':index['bytes'],'extra_members':len(contents),'physical_inputs':len(physical),'scoring_inputs':len(scoring),'jobs':len(plan['jobs']),'image_results':plan['expected_image_results'],'CSV_rows':plan['expected_CSV_rows']}))
