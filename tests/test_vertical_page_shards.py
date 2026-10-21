import copy
from pathlib import Path
import sys
import unittest
sys.path.insert(0,str(Path(__file__).resolve().parents[1]/'tools'))
from vertical_page_shards import partition_inputs, validate_coverage

class ShardTests(unittest.TestCase):
    def setUp(self):self.inputs=[{'id':f'i{i}'} for i in range(9)]
    def shard(self,index,total=4):
        ids=[r['id'] for i,r in enumerate(self.inputs) if i%total==index]
        return {'kind':'page100','complete':True,'physical_inputs':len(ids),'global_physical_inputs':9,
                'shard':{'index':index,'total':total,'physical_ids':ids}}
    def test_round_robin_retains_all_ids_and_their_original_order(self):
        parts=[partition_inputs(self.inputs,i,4) for i in range(4)]
        self.assertEqual([[r['id'] for r in p] for p in parts],[['i0','i4','i8'],['i1','i5'],['i2','i6'],['i3','i7']])
        self.assertEqual(partition_inputs(self.inputs),self.inputs)
        real=[{'id':str(i)} for i in range(921)]
        self.assertEqual([len(partition_inputs(real,i,4)) for i in range(4)],[231,230,230,230])
    def test_invalid_shards_cannot_silently_be_empty_or_unsharded(self):
        for index,total in [(0,0),(-1,4),(4,4),(None,4),(0,None),(True,4),(0,1.5),(0,10)]:
            with self.assertRaises(ValueError):partition_inputs(self.inputs,index,total)
    def test_complete_shards_merge_in_canonical_order(self):
        groups=validate_coverage([self.shard(i) for i in [2,0,3,1]],self.inputs)
        self.assertEqual([m['shard']['index'] for m in groups['page100']],[0,1,2,3])
        legacy={'kind':'page30','complete':True,'physical_inputs':9}
        self.assertEqual(validate_coverage([legacy],self.inputs),{'page30':[legacy]})
    def test_missing_duplicate_mixed_or_misbound_shards_are_rejected(self):
        good=[self.shard(i) for i in range(4)]
        variants=[good[:-1],good+[good[0]],good+[{'kind':'page100','complete':True,'physical_inputs':9}]]
        for field,value in [('complete',False),('physical_inputs',9),('global_physical_inputs',8)]:
            bad=copy.deepcopy(good);bad[0][field]=value;variants.append(bad)
        for field,value in [('total',3),('physical_ids',['i8','i4','i0'])]:
            bad=copy.deepcopy(good);bad[0]['shard'][field]=value;variants.append(bad)
        for records in variants:
            with self.assertRaises(ValueError):validate_coverage(records,self.inputs)
if __name__=='__main__':unittest.main()
