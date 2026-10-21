"""Prepare one frozen production-baseline diagnostic on additional real controls."""
import re
from vertical_page_matrix import safe_member


def image_manifest(master, root):
    inputs = master.get('inputs', [])
    scoring = master.get('scoring_inputs', [])
    if master.get('GT_frozen') is not True or not inputs or not scoring:
        raise ValueError('physical inputs and frozen GT are required')
    valid = lambda value: isinstance(value, str) and re.fullmatch(r'[A-Za-z0-9_-]+', value)
    ids = [row.get('id') for row in inputs]
    aliases = [row.get('id') for row in scoring]
    if (any(not valid(value) for value in ids + aliases)
            or len(set(ids)) != len(ids) or len(set(aliases)) != len(aliases)
            or {row.get('physical_id') for row in scoring} != set(ids)):
        raise ValueError('duplicate, missing or invalid physical/scoring input')
    return {'inputs':[{'id':row['id'], 'image':str(root / safe_member(row['image']))} for row in inputs]}


def text_diagnosis(truth, prediction):
    import unicodedata
    from verify_low_stop_evidence import metric_values
    keys=('char_errors','char_total','word_errors','word_total')
    score=lambda a,b:dict(zip(keys,metric_values(a,b,'none')))
    nfc=lambda text:unicodedata.normalize('NFC',text)
    return {
        'raw':score(truth,prediction),
        'nfc_diagnostic_only':score(nfc(truth),nfc(prediction)),
        'prediction_scalar_count':len(prediction),
        'prediction_ascii_spaces':prediction.count(' '),
        'prediction_nfc_changes':[{'scalar':c,'codepoint':f'U+{ord(c):04X}','nfc':nfc(c)}
                                  for c in dict.fromkeys(prediction) if nfc(c)!=c],
        'adoption_comparison':False,
    }
