import unicodedata

def verify_native_trace(rec,trace,text):
 assert trace['text']==text and trace['size']==rec['size']
 if rec['engine']=='freetype':
  assert rec['palt'] is None and trace['palt'] is None and trace['layout_engine']=='BASIC' and trace['face_index']==0
  assert trace['font']==rec['call']['font'] and rec['call']['text']==text and rec['call']['size']==rec['size']
  assert [x['character'] for x in trace['glyph_masks']]==list(dict.fromkeys(text))
  assert all(x['different_from_missing_glyph'] for x in trace['glyph_masks'])
  assert len(trace['prefix_lengths'])==len(text)+1
 else:
  assert rec['engine']=='coretext' and rec['exit_code']==0
  assert trace['palt']==rec['palt']==int(rec['argv'][3]) and rec['argv'][4]==text and float(rec['argv'][2])==trace['size']
  norm=lambda p:unicodedata.normalize('NFC',p)
  assert norm(trace['font_file'])==norm(rec['argv'][1])
  for run in trace['runs']:
   assert norm(run['font_file'])==norm(trace['font_file']) and run['font']==trace['font'] and 0 not in run['glyphs']
  assert sorted(i for run in trace['runs'] for i in run['string_indices'])==list(range(len(text)))
