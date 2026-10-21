"""Check the isolated C1 source and data frozen before C1 recognition."""
import gzip
import io
import json
from pathlib import Path
import zipfile
from vertical_page_matrix import safe_member,sha


def verify(folder):
    from PIL import Image,ImageChops,ImageDraw
    folder=Path(folder);index=json.loads((folder/'pure-before-nn-v1.json').read_text());archive=folder/safe_member(index['archive'])
    assert archive.stat().st_size==index['bytes'] and sha(archive.read_bytes())==index['sha256']
    with zipfile.ZipFile(archive) as z:
        assert len(z.namelist())==len(set(z.namelist())) and set(z.namelist())==set(index['members'])
        data={}
        for n,h in index['members'].items():
            safe_member(n);raw=z.read(n);assert len(raw)==h['bytes'] and sha(raw)==h['sha256'];data[n]=raw
    assert index['NN_calls_at_packet_creation']==0 and not index['production_hook'] and index['isolated_hook_connected']
    assert sha(data['evidence/generation-protocol.md'])==index['generation_protocol_sha256']
    assert sha(data['evidence/implementation-original-protocol.md'])==index['original_protocol_sha256']
    assert b'448 passed; 0 failed; 38 ignored' in data['evidence/refactor-full-default-v2.stdout']
    assert b'8 passed; 0 failed' in data['evidence/adapter-green.stdout']
    artifacts=0;binary_hashes=[]
    for variant,expected in index['source_files'].items():
        frozen=json.loads(data['evidence/'+variant+'-frozen.json']);assert frozen['source_files']==expected
        for n,h in expected.items():assert sha(data['sources/'+variant+'/'+n])==h
        raw=data['evidence/'+variant+'-build.jsonl'];assert sha(raw)==frozen['build_stdout_sha256']
        events=[json.loads(line) for line in raw.decode().splitlines() if line.startswith('{')]
        assert set(frozen['own_artifacts'])=={'lib','cli','test'}
        for key,event in frozen['own_artifacts'].items():
            assert event in events and event['fresh'] is False and 'onnx' in event['features'];artifacts+=1
        for key,binary in frozen['binaries'].items():assert binary['compiler_artifact']==frozen['own_artifacts'][key]
        binary_hashes.append(frozen['binaries']['cli']['sha256'])
    assert len(set(binary_hashes))==2
    registry_index=json.loads((folder/'known-inputs-before-nn-v1.index.json').read_text());path=folder/safe_member(registry_index['archive']);raw=path.read_bytes()
    assert len(raw)==registry_index['bytes'] and sha(raw)==registry_index['sha256']
    registry=json.loads(gzip.decompress(raw));known=registry['physical_inputs'];assert len(known)==1143 and len({r['id'] for r in known})==1143 and registry['C1_new_set_NN_calls']==0
    old={r[k] for r in known for k in ['image_sha256','rgb_sha256']}
    manifest=json.loads(data['evidence/new-manifest.json']);rows=manifest['inputs'];assert len(rows)==96 and len({r['id'] for r in rows})==96
    glyphs=0;new=[];cohorts={}
    for row in rows:
        values={}
        for key in ['image','rgb','truth']:
            raw=data['controls/inputs/'+Path(row[key]).name];assert sha(raw)==row[key+'_sha256'];values[key]=raw
        image=Image.open(io.BytesIO(values['image'])).convert('RGB');assert image.size==(row['width'],row['height']) and image.tobytes()==values['rgb']
        new.extend(row[k] for k in ['image_sha256','rgb_sha256']);cohort=row['cohort'];cohorts[cohort]=cohorts.get(cohort,0)+1
        text=values['truth'].decode()
        if 'glyph_boxes' in row:
            boxes=row['glyph_boxes'];assert len(boxes)==len(text);glyphs+=len(boxes);mask=Image.new('L',image.size,0);draw=ImageDraw.Draw(mask)
            for box in boxes:
                x0,y0,x1,y1=box;assert 0<=x0<x1<=image.width and 0<=y0<y1<=image.height
                crop=image.crop(box);assert ImageChops.difference(crop,Image.new('RGB',crop.size,'white')).getbbox() is not None
                draw.rectangle((x0,y0,x1-1,y1-1),fill=255)
            difference=ImageChops.difference(image,Image.new('RGB',image.size,'white'))
            assert ImageChops.multiply(difference,ImageChops.invert(mask).convert('RGB')).getbbox() is None
        elif cohort=='new_c1_portrait_horizontal':
            assert text.splitlines()==[row['row_text']]*row['rows'] and image.height>=8*image.width
        else:assert cohort=='new_c1_nonletter' and text==''
    assert sorted(cohorts.values())==[12,12,24,48] and glyphs==4860 and len(set(new))==192
    duplicates=len(set(new)&old);assert duplicates==0
    return {'complete':True,'controls':len(rows),'glyphs':glyphs,'known_physical_inputs':len(known),'duplicate_hashes':duplicates,'fresh_own_artifacts':artifacts,'production_hook':False,'NN_calls_at_packet_creation':0}


if __name__=='__main__':
    import argparse
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--folder',type=Path,default=Path(__file__).resolve().parents[1]/'docs/benchmarks/vertical-cli-model-agreement')
    args=parser.parse_args();print(json.dumps(verify(args.folder),indent=2))
