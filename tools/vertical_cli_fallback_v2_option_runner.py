"""Keep optional recognition behind the independently verified default gate."""

def require_new72_gate(verified):
    if (verified.get('complete') is not True or verified.get('matrix_gate_pass') is not True
            or verified.get('CLI_processes')!=576 or verified.get('verified_CSV_rows')!=1728
            or verified.get('regressions')!=0 or verified.get('raw_gain_pairs',0)<=0):
        raise ValueError('complete independently verified new72 nonregression and positive body gain are required')
