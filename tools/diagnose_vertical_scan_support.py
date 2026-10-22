"""Measure one direction-support ingredient; never invoke or select OCR output."""
from collections import Counter
import json
from pathlib import Path


def _bounds(mask, width, height):
    occupied=[i for i,v in enumerate(mask) if v]
    if not occupied:
        return None
    xs=[i % width for i in occupied];ys=[i // width for i in occupied]
    return [max(0,min(xs)-1),max(0,min(ys)-1),min(width,max(xs)+2),min(height,max(ys)+2)]


def _metrics(mask, width, bounds):
    if bounds is None:
        return None
    x0,y0,x1,y1=bounds;cross=x1-x0;runs=[];start=None;left=x1;right=x0
    for y in range(y0,y1+1):
        xs=[x for x in range(x0,x1) if y<y1 and mask[y*width+x]]
        if xs:
            if start is None:
                start=y
            left=min(left,min(xs));right=max(right,max(xs)+1)
        elif start is not None:
            runs.append((y-start,right-left));start=None;left=x1;right=x0
    if not runs:
        return None
    return {'support_bounds':bounds,'support_width':cross,'runs':len(runs),
            'anchors':sum(a*5>=cross*3 and b*5>=cross*3 for a,b in runs)}


def diagnose(rgb, width, height):
    if width<=0 or height<=0 or len(rgb)!=width*height*3:
        raise ValueError('invalid RGB dimensions')
    sums=[sum(rgb[i:i+3]) for i in range(0,len(rgb),3)]
    gray=[s//3 for s in sums];hist=Counter(gray);n=len(gray);total=sum(gray)
    count=moment=0;best=None;threshold=None
    for level in range(255):
        count+=hist[level];moment+=level*hist[level]
        if not count or count==n:
            continue
        num=(total*count-moment*n)**2;den=count*(n-count)
        if best is None or num*best[1]>best[0]*den:
            best=(num,den);threshold=level
    original=_bounds([s<765 for s in sums],width,height)
    mask=[g<=threshold for g in gray] if threshold is not None else None
    return {'dimensions':[width,height],'gray_range':[min(gray),max(gray)],
            'fixed220':_metrics([s<660 for s in sums],width,original),
            'otsu_threshold':threshold,
            'otsu':_metrics(mask,width,_bounds(mask,width,height)) if mask is not None else None}


def main():
    from verify_additional_real_baseline import _packet
    from vertical_page_matrix import sha
    root=Path(__file__).resolve().parents[1]
    index,data=_packet(root/'docs/benchmarks/real-vertical-additional-controls','index.json')
    manifest=json.loads(data['tmp/real-vertical-additional-controls/additional-controls-manifest-v1.json'])
    rows=[]
    for row in manifest['inputs']:
        rgb=data[row['rgb']];assert sha(rgb)==row['rgb_sha256']
        rows.append({'id':row['id'],'rgb_sha256':row['rgb_sha256'],**diagnose(rgb,row['width'],row['height'])})
    print(json.dumps({'controls_sha256':index['sha256'],'new_NN_calls':0,
                      'known_scope_only':True,'Rust_adoption_trace':False,'rows':rows},indent=2))


if __name__=='__main__':
    main()
