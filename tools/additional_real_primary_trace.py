"""Prepare the six-image diagnostic without passing GT to the frozen observer."""
from additional_real_diagnostic import image_manifest


def trace_batches(master,root):
    manifest=image_manifest(master,root)
    if len(manifest['inputs'])!=6:
        raise ValueError('the fixed diagnostic requires exactly six physical images')
    return [{'inputs':manifest['inputs'][offset:offset+3]} for offset in [0,3]]


def same_page_output(trace,baseline):
    import struct
    try:
        actual={'id':trace['id'],'text':trace['text'],'page_index':trace['page_index'],
                'lines':[{'bbox':r['bbox'],'text':r['text'],'mean_bits':r['mean_bits']} for r in trace['lines']]}
        expected={'id':baseline['id'],'text':baseline['text'],'page_index':baseline['page_index'],
                  'lines':[{'bbox':r['bbox'],'text':r['text'],'mean_bits':struct.unpack('!I',struct.pack('!f',r['confidence']))[0]} for r in baseline['lines']]}
        return actual==expected
    except (KeyError,TypeError,ValueError,OverflowError,struct.error):
        return False
