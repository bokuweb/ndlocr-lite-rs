"""Build lossless, deduplicated primary-input diagnostics without selecting by GT."""
import hashlib
from PIL import Image


def valid_rgb(rgb,width,height):
    return (type(width) is int and type(height) is int and width>0 and height>0
            and isinstance(rgb,bytes) and len(rgb)==width*height*3)


def rotate_primary(rgb,width,height):
    if not valid_rgb(rgb,width,height):raise ValueError('invalid RGB dimensions')
    image=Image.frombytes('RGB',(width,height),rgb).transpose(Image.Transpose.ROTATE_90)
    return image.tobytes(),image.width,image.height


def unique_primary_inputs(rows):
    unique={};origins=set()
    for row in rows:
        origin=row['origin'];model=row['model'];rgb=row['rgb'];width=row['width'];height=row['height']
        if (not isinstance(origin,str) or not origin or origin in origins or model not in ['30','50','100']
                or not valid_rgb(rgb,width,height) or not isinstance(row['raw'],str)
                or type(row['mean_bits']) is not int or not 0<=row['mean_bits']<2**32):
            raise ValueError('invalid or duplicate primary input')
        origins.add(origin);digest=hashlib.sha256(rgb).hexdigest();key=(model,width,height,digest)
        if key not in unique:
            ident=hashlib.sha256(f'{model}:{width}:{height}:{digest}'.encode()).hexdigest()
            unique[key]={k:row[k] for k in ['model','width','height','rgb','raw','mean_bits']}
            unique[key].update(id=ident,origins=[],rgb_sha256=digest)
        stored=unique[key]
        if stored['raw']!=row['raw'] or stored['mean_bits']!=row['mean_bits']:raise ValueError('conflicting raw result for identical model input')
        stored['origins'].append(origin)
    if not unique:raise ValueError('empty primary scope')
    return list(unique.values())
