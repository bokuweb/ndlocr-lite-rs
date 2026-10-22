import copy
import hashlib
import json
from pathlib import Path
import sys
import tempfile
import unittest
import zipfile
sys.path.insert(0,str(Path(__file__).resolve().parents[1]/'tools'))
from verify_low_stop_evidence import Packet, metric_values, verify_csv, cut_glyphs

class PacketTests(unittest.TestCase):
    def setUp(self):
        self.temp=tempfile.TemporaryDirectory();self.root=Path(self.temp.name)
        data=b'fixed observation';self.digest=hashlib.sha256(data).hexdigest()
        self.archive=self.root/'data.zip'
        with zipfile.ZipFile(self.archive,'w') as z:z.writestr('objects/'+self.digest,data)
        self.index={'schema':1,'archive':'data.zip','sha256':hashlib.sha256(self.archive.read_bytes()).hexdigest(),'bytes':self.archive.stat().st_size,'logical_files':{'tmp/case.json':{'sha256':self.digest,'bytes':len(data)}},'backing_objects':{self.digest:{'member':'objects/'+self.digest,'bytes':len(data)}}}
    def tearDown(self):self.temp.cleanup()
    def write_index(self,value):
        p=self.root/'index.json';p.write_text(json.dumps(value));return p
    def test_reads_only_declared_and_verified_logical_bytes(self):
        packet=Packet(self.write_index(self.index));self.assertEqual(packet.read('tmp/case.json'),b'fixed observation')
        with self.assertRaises(ValueError):packet.read('../outside')
    def test_rejects_corrupt_archive_or_wrong_logical_size(self):
        for kind in ('sha','bytes'):
            bad=copy.deepcopy(self.index)
            if kind=='sha':bad['sha256']='0'*64
            else:bad['logical_files']['tmp/case.json']['bytes']+=1
            with self.assertRaises(ValueError):Packet(self.write_index(bad))
    def test_rejects_missing_or_unsafe_members_and_unreferenced_objects(self):
        for kind in ('missing','unsafe','unreferenced'):
            bad=copy.deepcopy(self.index)
            if kind=='missing':bad['backing_objects'][self.digest]['member']='objects/missing'
            elif kind=='unsafe':bad['logical_files']['../bad']=bad['logical_files'].pop('tmp/case.json')
            else:bad['logical_files']={}
            with self.assertRaises(ValueError):Packet(self.write_index(bad))
class MetricTests(unittest.TestCase):
    def test_width_folds_only_ascii_and_keeps_compatibility_forms(self):
        self.assertEqual(metric_values('Ａ①','A①','none'), (1,2,1,1))
        self.assertEqual(metric_values('Ａ①','A①','width'), (0,2,0,1))
        self.assertEqual(metric_values('Ａ①','A1','width'), (1,2,1,1))
    def test_strict_keeps_japanese_punctuation_and_empty_truth_has_unit_denominator(self):
        self.assertEqual(metric_values('Ａ !\r\n漢、','Ａ\n漢','strict'), (1,4,1,2))
        self.assertEqual(metric_values('','A','none'), (1,1,1,1))

class CsvTests(unittest.TestCase):
    def setUp(self):
        self.expected={('candidate','a.txt'):(0,2,0,1),('baseline','a.txt'):(1,2,1,1)}
        fields=('char_errors','char_total','word_errors','word_total')
        self.rows=[dict(system=k[0],file=k[1],**dict(zip(fields,map(str,v)))) for k,v in self.expected.items()]
    def test_complete_metrics_cannot_be_replaced_by_partial_or_duplicate_rows(self):
        self.assertEqual(verify_csv(self.rows,self.expected),2)
        for rows in [self.rows[:1],self.rows+[self.rows[0]]]:
            with self.assertRaises(ValueError):verify_csv(rows,self.expected)
    def test_every_error_and_denominator_is_independently_required(self):
        for field in ['char_errors','char_total','word_errors','word_total']:
            rows=copy.deepcopy(self.rows);rows[0][field]=str(int(rows[0][field])+1)
            with self.assertRaises(ValueError):verify_csv(rows,self.expected)

class GeometryTests(unittest.TestCase):
    def test_glyph_edges_are_safe_but_any_internal_boundary_is_a_cut(self):
        boxes=[[1,10,8,20],None,[2,30,9,40]]
        self.assertEqual(cut_glyphs(boxes,[0,10,20,30,40,50]),[])
        self.assertEqual(cut_glyphs(boxes,[15,35]),[(0,15),(2,35)])

if __name__=='__main__':unittest.main()
