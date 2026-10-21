from pathlib import Path
import sys
import unittest
sys.path.insert(0,str(Path(__file__).resolve().parents[1]/'tools'))


class AdditionalRealPrimaryTraceTest(unittest.TestCase):
    def manifest(self,n=6):
        return {'GT_frozen':True,'inputs':[{'id':'col'+str(i),'image':'images/'+str(i)+'.png','truth':'NEVER_TO_NN'} for i in range(n)],
                'scoring_inputs':[{'id':'gt'+str(i),'physical_id':'col'+str(i),'truth':'ALSO_SECRET'} for i in range(n)]}

    def test_two_three_input_batches_keep_every_id_once_and_omit_gt(self):
        from additional_real_primary_trace import trace_batches
        rows=trace_batches(self.manifest(),Path('/root'))
        self.assertEqual([len(r['inputs']) for r in rows],[3,3])
        self.assertEqual([x['id'] for r in rows for x in r['inputs']],['col'+str(i) for i in range(6)])
        for r in rows:
            for x in r['inputs']:self.assertEqual(set(x),{'id','image'})

    def test_a_different_count_or_unfrozen_gt_does_not_silently_change_the_observation(self):
        from additional_real_primary_trace import trace_batches
        for n in [0,3,5,7,9]:
            with self.subTest(n=n),self.assertRaises(ValueError):trace_batches(self.manifest(n),Path('/root'))
        manifest=self.manifest();manifest['GT_frozen']=False
        with self.assertRaises(ValueError):trace_batches(manifest,Path('/root'))

class PageTraceAlignmentTest(unittest.TestCase):
    def test_alignment_requires_literal_text_boxes_line_count_and_actual_f32_bits(self):
        from additional_real_primary_trace import same_page_output
        import copy,struct
        bits=struct.unpack('!I',struct.pack('!f',0.99))[0]
        base={'id':'a','text':'A\nB','page_index':0,'lines':[{'bbox':[0,1,2,3],'text':'A','confidence':0.99},{'bbox':[2,3,4,5],'text':'B','confidence':0.99}]}
        trace={'id':'a','text':'A\nB','page_index':0,'lines':[{'bbox':[0,1,2,3],'text':'A','mean_bits':bits},{'bbox':[2,3,4,5],'text':'B','mean_bits':bits}]}
        self.assertTrue(same_page_output(trace,base))
        for change in ['text','bbox','bits','lines','id']:
            value=copy.deepcopy(trace)
            if change=='text':value['text']='A B'
            if change=='bbox':value['lines'][0]['bbox'][0]=1
            if change=='bits':value['lines'][0]['mean_bits']+=1
            if change=='lines':value['lines'].pop()
            if change=='id':value['id']='other'
            with self.subTest(change=change):self.assertFalse(same_page_output(value,base))
