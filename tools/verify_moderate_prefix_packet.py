"""Replay the pre-recognition R controls without fonts, weights or inference."""
from collections import Counter
import gzip
import io
import json
from pathlib import Path
import zipfile
from PIL import Image, ImageChops
from moderate_prefix_controls import control_plan, PARAGRAPH
from verify_additional_real_baseline import _packet
from vertical_page_matrix import sha


def verify_route(image,parts,read,glyph_boxes):
    width,height=image.size;rotated=image.transpose(Image.Transpose.ROTATE_90);cursor=0;pixels=0
    assert parts
    for part in parts:
        left,right=part['range']
        assert type(left) is int and type(right) is int and left==cursor and left<right<=height
        assert (part['width'],part['height'])==(right-left,width) and part['space_before'] is False
        assert all(not (top<right<bottom) for _,top,_,bottom in glyph_boxes)
        assert read(part['file'])==rotated.crop((left,0,right,width)).tobytes()
        pixels+=(right-left)*width;cursor=right
    assert cursor==height
    return pixels


def verify(folder):
    folder=Path(folder);index,data=_packet(folder,'nn0-v1.index.json')
    pi,prior=_packet(folder.parent/'real-vertical-primary-partition-probe','before-nn-v1.index.json')
    vi,v2=_packet(folder.parent/'vertical-cli-fallback-v2','before-nn-v1.index.json')
    si,stop=_packet(folder.parent/'real-vertical-additional-stop-probe','before-nn-v1.index.json')
    assert index['geometry_source_packet_SHA256']==pi['sha256']
    frozen=json.loads(prior['evidence/geometry-binary-frozen-v1.json']);bridge=json.loads(prior['evidence/source-bridge-v1.json'])
    assert bridge['packet_SHA256']=={'FVCv2':vi['sha256'],'stop':si['sha256']}
    sources={name:{'FVCv2':v2,'stop':stop,'this':prior}[m['packet']][m['member']] for name,m in bridge['source_files'].items()}
    assert {n:sha(b) for n,b in sources.items()}==frozen['source_files']
    artifact=frozen['compiler_artifact'];log=prior['TDD/partition-green-v2.jsonl']
    assert sha(log)==frozen['build_stdout_sha256'] and artifact['fresh'] is False and 'onnx' not in artifact['features']
    assert artifact in [json.loads(line) for line in log.decode().splitlines() if line.startswith('{')]
    root=Path(frozen['binary_path']).parents[2];prefix='tmp/vertical-moderate-prefix/'
    command=json.loads(data['evidence/geometry-command-v1.json']);before=json.loads(data['evidence/geometry-before-v1.json'])
    for absolute,digest in command['files_sha256'].items():
        name=str(Path(absolute).relative_to(root))
        if name in sources:raw=sources[name]
        elif name=='tmp/real-vertical-primary-partition-probe/geometry-binary':assert digest==frozen['binary_sha256'];continue
        elif name=='tmp/real-vertical-primary-partition-probe/geometry-binary-frozen-v1.json':raw=prior['evidence/geometry-binary-frozen-v1.json']
        elif name.startswith(prefix):raw=data['evidence/'+name[len(prefix):]]
        else:raw=data[name]
        assert sha(raw)==digest,name
    assert command['complete'] and command['exit_code']==0 and command['frozen_files_after_unchanged']
    assert not before['complete'] and all(command[k]==v for k,v in before.items() if k!='complete')
    assert command['NN_calls']==command['model_loads']==0
    assert command['argv']==[frozen['binary_path'],'--ignored','--exact','infer::vertical_capacity::primary_partition_geometry::record_all_primary_partitions_without_nn','--nocapture','--test-threads=1']
    assert command['env']=={'NDLOCR_PRIMARY_PARTITION_MANIFEST':str(root/prefix/'geometry-manifest-v1.json'),'NDLOCR_PRIMARY_PARTITION_OUTPUT':str(root/prefix/'geometry-v1')}
    config=json.loads(data['evidence/generation-before-render-v2.json']);rows=json.loads(data['evidence/new-manifest-v2.json'])['inputs'];plan=control_plan(['arial','bold','serif'])
    assert config['plan']==plan and config['paragraph']==PARAGRAPH and len(rows)==72
    for field,name in [('generator','evidence/generate_v2.py'),('helper','tools/moderate_prefix_controls.py'),('spec','docs/vertical_moderate_prefix_proposal.md'),('charset','models/NDLmoji.yaml')]:assert config[field+'_sha256']==sha(data[name])
    assert config['NN_calls']==config['model_loads']==0 and config['R_policy_started'] is False
    charset_line,=[line for line in data['models/NDLmoji.yaml'].decode().splitlines() if line.startswith('  charset_train: ')]
    charset=json.loads(charset_line.split(': ',1)[1]);assert isinstance(charset,str)
    old=json.loads((folder.parent/'vertical-cli-fallback-v2/full-matrix-v1/bundle.json').read_text());assert len(old['physical_inputs'])==1311
    assert sha(data['models/NDLmoji.yaml'])==old['assets']['charset']['sha256']
    old_hashes={r[k] for r in old['physical_inputs'] for k in ['image_sha256','rgb_sha256']};images={};glyphs=0;hashes=[]
    with zipfile.ZipFile(io.BytesIO(data['evidence/geometry-original-RGB-v1.zip'])) as rgb:
        assert set(rgb.namelist())=={r['id']+'.rgb' for r in rows}
        for row,wanted in zip(rows,plan,strict=True):
            assert all(row[k]==v for k,v in wanted.items())
            ident=row['id'];png=data['inputs/'+ident+'.png'];truth=data['inputs/'+ident+'.txt'];raw=rgb.read(ident+'.rgb')
            assert sha(png)==row['image_sha256'] and sha(truth)==row['truth_sha256'] and sha(raw)==row['rgb_sha256']
            assert truth.decode()==row['text'] and set(c for c in row['text'] if not c.isspace())<=set(charset)
            image=Image.open(io.BytesIO(png)).convert('RGB');assert image.size==(row['width'],row['height']) and image.tobytes()==raw
            hashes.extend([sha(png),sha(raw)]);images[ident]=image
            boxes=row.get('glyph_boxes',[])
            if row['cohort'] in ['body','repeated']:
                assert len(boxes)==len(row['text'])
                for i,box in enumerate(boxes):
                    top=12+row['padding']+52*i;cell=image.crop((0,top,image.width,top+52));ink=ImageChops.difference(cell,Image.new('RGB',cell.size,'white')).getbbox();assert ink
                    assert list(ink)==[box[0],box[1]-top,box[2],box[3]-top];glyphs+=1
            else:assert not boxes
    assert glyphs==5256 and len(set(hashes))==144 and not old_hashes.intersection(hashes)
    generation=json.loads(data['evidence/generation-complete-v2.json']);assert generation['new_inputs']==72 and generation['cohorts']==dict(Counter(r['cohort'] for r in rows)) and generation['independently_reconstructed_glyphs']==glyphs
    assert generation['PNG_RGB_exact'] and generation['known1311_hash_overlap']==0 and generation['model_charset_covers_all_declared_nonspace_scalars'] and generation['font_missing_glyph_signature_rejected']
    stopped_png=stopped_rgb=partial_rgb=0
    for name,raw in data.items():
        if name.startswith('stopped-inputs/') and name.endswith('.png'):
            assert raw==data['inputs/'+Path(name).name];stopped_png+=1
        if name.startswith('stopped-inputs/') and name.endswith('.rgb.gz'):
            ident=Path(name).name[:-7];original=images[ident].tobytes();partial=gzip.decompress(raw)
            # ENOSPC left one zero-byte RGB file; preserve that empty prefix too.
            assert original.startswith(partial)
            if partial==original:stopped_rgb+=1
            else:partial_rgb+=1
    assert (stopped_png,stopped_rgb,partial_rgb)==(69,68,1)
    refs=json.loads(data['evidence/geometry-references-v1.json']);manifest=json.loads(data['evidence/geometry-manifest-v1.json'])['items'];records=json.loads(data['evidence/geometry-v1/records.json'])
    assert len(refs)==len(manifest)==len(records)==216
    pixels=parts=0;members=[];physical={r['id']:r for r in rows};pairs=[]
    with zipfile.ZipFile(io.BytesIO(data['evidence/geometry-v1/parts.zip'])) as archive:
        for ref,item,record in zip(refs,manifest,records,strict=True):
            ident=ref['id'];source=physical[ref['physical_id']];model=ref['model'];pairs.append((model,source['id']))
            assert ident==sha(f'{model}:{source["id"]}'.encode()) and ref['glyph_boxes']==source.get('glyph_boxes')
            assert item=={'id':ident,'archive':str(root/prefix/'geometry-original-RGB-v1.zip'),'member':source['id']+'.rgb','width':source['width'],'height':source['height'],'input':[{'30':256,'50':384,'100':768}[model],24]}
            assert record['id']==ident and record['error'] is None and record['width']==source['width'] and record['height']==source['height'] and record['input']==item['input'] and len(record['routes'])==2
            for route in record['routes']:
                pixels+=verify_route(images[source['id']],route,archive.read,source.get('glyph_boxes',[]));parts+=len(route);members.extend(p['file'] for p in route)
        assert len(members)==len(set(members)) and set(members)==set(archive.namelist())
    assert set(pairs)=={(model,r['id']) for model in ['30','50','100'] for r in rows} and len(pairs)==len(set(pairs))
    result={'physical_inputs':72,'geometry_inputs':216,'part_RGBs':parts,'independent_pixels':pixels,'glyph_cut_events':0,'NN_calls':0,'R_policy_started':False}
    saved=json.loads(data['evidence/geometry-audit-v1.json']);assert not saved['cuts'] and not saved['errors']
    assert all(saved['summary'][k]==v for k,v in result.items()) and saved['summary']['R_geometry_gate_pass']
    assert parts==2952 and pixels==186559200 and index['NN_calls']==index['model_loads']==generation['NN_calls']==generation['model_loads']==0 and index['R_policy_started'] is generation['R_policy_started'] is False
    return result


if __name__=='__main__':
    print(json.dumps(verify(Path(__file__).resolve().parents[1]/'docs/benchmarks/vertical-moderate-prefix'),indent=2))
