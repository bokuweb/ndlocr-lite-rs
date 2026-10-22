"""Extend the existing complete gate without losing origin or GT aliases."""
from copy import deepcopy
from vertical_cli_fallback_full_matrix import merge_controls


def combine_scope(previous,new):
    old_images=previous.get('physical_inputs',[]);old_scores=previous.get('scoring_inputs',[]);extra=new.get('inputs',[])
    if len(old_images)!=1239 or len(old_scores)!=1245 or len(extra)!=72:
        raise ValueError('the complete v1 controls and all v2 new72 are required')
    old_rgb={r.get('rgb_sha256') for r in old_images};new_rgb=[r.get('rgb_sha256') for r in extra]
    if None in old_rgb or None in new_rgb or len(set(new_rgb))!=72 or old_rgb&set(new_rgb):
        raise ValueError('v2 controls must have distinct new RGBs')
    images=deepcopy(old_images);scores=deepcopy(old_scores)
    for row in extra:
        if any(k not in row for k in ['id','image','image_sha256','rgb_sha256','width','height','cohort','truth','truth_sha256']):
            raise ValueError('incomplete v2 physical or scoring input')
        images.append({k:deepcopy(row[k]) for k in ['id','image','image_sha256','rgb_sha256','width','height','cohort']}|{'origin':'fvc_v2_new72'})
        scores.append({'id':row['id'],'physical_id':row['id'],'truth':row['truth'],'truth_sha256':row['truth_sha256'],'origin':'fvc_v2_new72'})
    origins=list(dict.fromkeys(r.get('origin') for r in images))
    groups=[{'origin':origin,'inputs':[r for r in images if r.get('origin')==origin],
             'scoring_inputs':[r for r in scores if r.get('origin')==origin]} for origin in origins]
    merge_controls(groups)
    if len({(r['width'],r['height'],r['rgb_sha256']) for r in images})!=1310:
        raise ValueError('unexpected global RGB population')
    return images,scores
