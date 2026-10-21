"""The fixed R gate, matching the existing eight-route native collector."""
import re
from moderate_prefix_controls import control_plan

ROUTES=('raw_auto','raw_horizontal','normal_auto','normal_horizontal','fragment_auto','fragment_horizontal','fragment_batch_auto','fragment_exclusive_batch_auto')
AFFECTED=('fragment_auto','fragment_batch_auto','fragment_exclusive_batch_auto')


def matrix_scope(ids):
    declared=[row['id'] for row in control_plan(['arial','bold','serif'])]
    if len(ids)!=72 or any(not isinstance(i,str) or not re.fullmatch(r'[A-Za-z0-9_-]+',i) for i in ids) or list(ids)!=declared:
        raise ValueError('the entire fixed 72-input scope is required')
    return {'input_ids':list(ids),'models':['30','50','100'],'variants':['baseline','candidate'],'routes':list(ROUTES),'affected_routes':list(AFFECTED),'unchanged_routes':[r for r in ROUTES if r not in AFFECTED],'outer_image_API_calls':(len(ids)*6+2)*3*2,'image_results':len(ids)*8*3*2,'CSV_rows':len(ids)*8*3*2*3,'raw_comparison_pairs':len(ids)*8*3,'normalized_comparison_pairs':len(ids)*8*3*3,'explicit_model_loads':3*3*2,'internal_session_run_calls':None}
