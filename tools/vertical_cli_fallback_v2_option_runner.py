"""Keep recognition and integration behind independently verified frozen gates."""

def require_new72_gate(verified):
    if (verified.get('complete') is not True or verified.get('matrix_gate_pass') is not True
            or verified.get('CLI_processes')!=576 or verified.get('verified_CSV_rows')!=1728
            or verified.get('regressions')!=0 or verified.get('raw_gain_pairs',0)<=0):
        raise ValueError('complete independently verified new72 nonregression and positive body gain are required')


def require_root_v2_full_gate(result):
    expected={'image_results':26220,'verified_CSV_rows':79020,'regression_pairs':0,'page_body_bbox_confidence_changes':0}
    if (result.get('complete') is not True or result.get('full_gate_pass') is not True
            or any(type(result.get(key)) is not int or result[key]!=value for key,value in expected.items())):
        raise ValueError('Root integration requires the complete nonregressing frozen FVC v2 gate')
