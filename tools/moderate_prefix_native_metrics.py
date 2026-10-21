"""Independent pair classification for the fixed R native gate."""
import math
import struct
from moderate_prefix_matrix_scope import ROUTES, AFFECTED
from verify_low_stop_evidence import metric_values


def pair_decision(before,after,truth,route,cohort):
    if route not in ROUTES or cohort not in ['body','repeated','portrait','nonletter']:raise ValueError('unknown scope')
    for value in [before,after]:
        if set(value)!={'text','mean_bits'} or not isinstance(value['text'],str) or type(value['mean_bits']) is not int or not 0<=value['mean_bits']<2**32:raise ValueError('invalid result')
        mean=struct.unpack('!f',struct.pack('!I',value['mean_bits']))[0]
        if not math.isfinite(mean) or not 0<=mean<=1:raise ValueError('invalid confidence')
    same=before['text']==after['text'];same_mean=before['mean_bits']==after['mean_bits'];metrics={mode:{'before':metric_values(truth,before['text'],mode),'after':metric_values(truth,after['text'],mode)} for mode in ['none','width','strict']}
    regression=any(r['after'][i]>r['before'][i] for r in metrics.values() for i in [0,2]);raw=metrics['none']
    return {'body_unchanged':same,'confidence_bits_unchanged':same_mean,'protected_unchanged':(route in AFFECTED or (same and same_mean)) and (cohort=='body' or same),'regression':regression,'raw_body_gain':cohort=='body' and route=='fragment_auto' and raw['after'][0]<raw['before'][0] and raw['after'][2]<=raw['before'][2],'metrics':metrics}
