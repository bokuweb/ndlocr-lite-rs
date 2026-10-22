from pathlib import Path
import sys
import unittest
import struct
sys.path.insert(0,str(Path(__file__).resolve().parents[1]/'tools'))


class AdditionalStopProbeTest(unittest.TestCase):
    def test_decoder_replay_rejects_emission_after_an_earlier_stop_condition(self):
        from verify_additional_real_stop_probe import verify_decode_steps
        bits=lambda p:struct.unpack('!I',struct.pack('!f',p))[0]
        def step(i,character,p,start):
            return {'timestep':i,'text_range':None if start is None else [start,start+len(character.encode())],'candidates':[{'token_id':0 if character is None else 1,'character':character,'probability':p,'probability_bits':bits(p)}]}
        rows=[{'text':'甲','mean_bits':bits(1.0),'stop_reason':'EndOfSequence','steps':[step(0,None,1.0,None),step(1,'甲',1.0,0),step(2,None,1.0,None)]},
              {'text':'甲乙丙','mean_bits':bits(.7),'stop_reason':'EndOfSequence','steps':[step(0,'甲',1.0,0),step(1,'乙',1.0,3),step(2,'丙',.1,6),step(3,None,1.0,None)]}]
        for row in rows:
            with self.subTest(row=row),self.assertRaises(AssertionError):verify_decode_steps(row)

    def test_saved_stop_probe_matches_all_original_raws_and_reports_actual_stop_events(self):
        from verify_additional_real_stop_probe import verify
        result=verify(Path(__file__).resolve().parents[1]/'docs/benchmarks/real-vertical-additional-stop-probe')
        self.assertEqual(result['raw_API_calls'],18)
        self.assertEqual(result['original_routes_matched'],36)
        self.assertEqual(result['raw_or_confidence_mismatches'],0)
        self.assertEqual(result['stop_counts'],{'EndOfSequence':17,'LowConfidence':1})
        self.assertEqual(result['model_session_loads'],3)
        self.assertEqual(result['new_model_calls_in_replay'],0)
        self.assertFalse(result['new_accuracy_gain'])

    def row(self,origin='30_eval_a',model='30',**overrides):
        return dict({'origin':origin,'model':model,'width':2,'height':3,'rgb':bytes(range(18)),'raw':'甲乙 ','mean_bits':1065353216},**overrides)

    def test_dedup_preserves_every_origin_and_keeps_different_models_separate(self):
        from additional_real_stop_probe import unique_primary_inputs
        result=unique_primary_inputs([self.row(),self.row('30_default_a'),self.row('50_eval_a','50')])
        self.assertEqual(len(result),2)
        self.assertEqual(result[0]['origins'],['30_eval_a','30_default_a'])
        self.assertEqual(result[1]['origins'],['50_eval_a'])
        self.assertEqual(result[0]['raw'],'甲乙 ')
        self.assertNotEqual(result[0]['id'],result[1]['id'])

    def test_conflicting_raws_duplicate_origins_and_invalid_rgb_fail_before_inference(self):
        from additional_real_stop_probe import unique_primary_inputs
        for rows in [[self.row(),self.row('other',raw='別')],[self.row(),self.row('other',mean_bits=0)],[self.row(),self.row()],[self.row(rgb=b'')],[self.row(width=0)],[self.row(model='unknown')]]:
            with self.subTest(rows=rows),self.assertRaises(ValueError):unique_primary_inputs(rows)

    def test_rotated_rgb_uses_the_same_lossless_coordinates_for_varied_dimensions(self):
        from additional_real_stop_probe import rotate_primary
        for width in range(1,7):
            for height in range(1,10):
                rgb=bytes((i%256 for i in range(width*height*3)))
                rotated,w,h=rotate_primary(rgb,width,height)
                self.assertEqual((w,h),(height,width))
                for y in range(height):
                    for x in range(width):
                        a=(y*width+x)*3;b=((width-1-x)*height+y)*3
                        self.assertEqual(rotated[b:b+3],rgb[a:a+3])
