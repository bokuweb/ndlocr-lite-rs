"""Fixed nondefault-option Cartesian scope; no recognition or GT selection."""
from itertools import product

SELECTED=(
    'fvc2_arial_49_pad6','fvc2_arial_89_pad18','fvc2_bold_69_pad6','fvc2_serif_109_pad18',
    'fvc2_repeat_arial_0_pad6','fvc2_portrait_serif_1_32',
    'fvc2_shape_white_broken_octagon_64','fvc2_shape_warm_offset_kite_124',
)


def option_plan(inputs):
    expected={f'fvc2_{font}_{length}_pad{padding}' for font,length,padding in product(['arial','bold','serif'],[49,69,89,109,129,149],[6,18])}
    expected|={f'fvc2_repeat_{font}_{pattern}_pad{padding}' for font,pattern,padding in product(['arial','bold','serif'],[0,1],[6,18])}
    expected|={f'fvc2_portrait_{font}_{pattern}_{rows}' for font,pattern,rows in product(['arial','bold','serif'],[0,1],[32,42])}
    expected|={f'fvc2_shape_{background}_{pattern}_{groups}' for background,pattern,groups in product(['white','warm','gray'],['broken_octagon','offset_kite'],[64,124])}
    ids=[r.get('id') for r in inputs]
    if len(ids)!=72 or len(set(ids))!=72 or set(ids)!=expected:
        raise ValueError('the complete fixed v2 72-input set is required')
    default={'line_crop_padding':0,'min_line_confidence':0.3,'binarize_threshold':220,'quality_boost':False,'quality_boost_min_delta':0.03,'cascade_threshold_30_to_50':25,'cascade_threshold_50_to_100':45,'split_long_lines':True,'split_long_line_char_threshold':80}
    profiles={'padding5':{'line_crop_padding':5},'reject_all':{'min_line_confidence':1.01},'threshold180':{'binarize_threshold':180},'threshold240':{'binarize_threshold':240},'quality':{'quality_boost':True},'custom':{'line_crop_padding':13,'min_line_confidence':0.0,'binarize_threshold':210,'quality_boost':True,'quality_boost_min_delta':0.01,'cascade_threshold_30_to_50':125,'cascade_threshold_50_to_100':200,'split_long_lines':False}}
    rows={r['id']:r for r in inputs}
    return [{'variant':variant,'profile':profile,'id':ident,'image':rows[ident]['image'],'deim':deim,'cascade':cascade,'options':default|profiles[profile]}
            for variant,profile,ident,deim,cascade in product(['baseline','candidate'],profiles,SELECTED,[True,False],[True,False])]


def option_command(row,binary,models,assets,body):
    from vertical_page_matrix import cli_command
    argv=cli_command(binary,row['image'],models,assets,'deim' if row['deim'] else 'bands',body)
    argv+=['--enable-cascade',str(row['cascade']).lower()]
    for name,value in row['options'].items():
        if name=='quality_boost':
            if value:argv+=['--quality-boost']
        else:argv+=['--'+name.replace('_','-'),str(value).lower()]
    return argv+['--structure-rules','true']
