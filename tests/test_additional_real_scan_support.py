from pathlib import Path
import json
import sys
import unittest
sys.path.insert(0,str(Path(__file__).resolve().parents[1]/'tools'))

class ScanSupportTest(unittest.TestCase):
    def test_uniform_and_invalid_inputs_have_no_invented_threshold(self):
        from diagnose_vertical_scan_support import diagnose
        self.assertIsNone(diagnose(bytes([200])*12*96*3,12,96)['otsu_threshold'])
        self.assertIsNone(diagnose(bytes([255])*12*96*3,12,96)['fixed220'])
        for rgb,w,h in [(b'',0,1),(b'',1,1),(b'123',-1,1),(b'1234',1,1)]:
            with self.assertRaises(ValueError):diagnose(rgb,w,h)

    def test_support_mask_separates_gray_paper_but_does_not_modify_the_pixels(self):
        from diagnose_vertical_scan_support import diagnose
        w,h=20,248;rgb=bytearray([200]*w*h*3)
        for g in range(12):
            for y in range(4+20*g,20+20*g):
                for x in range(2,18):rgb[(y*w+x)*3:(y*w+x)*3+3]=b'\0\0\0'
        before=bytes(rgb);v=diagnose(rgb,w,h)
        self.assertEqual(v['fixed220']['anchors'],1)
        self.assertEqual(v['otsu_threshold'],0)
        self.assertEqual(v['otsu']['anchors'],12)
        self.assertEqual(v['otsu']['support_bounds'],[1,3,19,241])
        self.assertEqual(bytes(rgb),before)

    def test_otsu_floors_the_rgb_average_and_keeps_the_first_tie(self):
        from diagnose_vertical_scan_support import diagnose
        self.assertEqual(diagnose(bytes([63,64,65,199,200,201]),1,2)['otsu_threshold'],64)

    def test_the_frozen_real_six_do_not_support_the_mask_only_proposal(self):
        from diagnose_vertical_scan_support import diagnose
        from verify_additional_real_baseline import _packet
        root=Path(__file__).resolve().parents[1]
        _,data=_packet(root/'docs/benchmarks/real-vertical-additional-controls','index.json')
        rows=json.loads(data['tmp/real-vertical-additional-controls/additional-controls-manifest-v1.json'])['inputs']
        values=[diagnose(data[r['rgb']],r['width'],r['height']) for r in rows]
        self.assertEqual(len(values),6)
        self.assertTrue(all(v['otsu']['anchors']<6 for v in values))
        self.assertEqual([v['fixed220']['anchors'] for v in values[3:]],[1,2,1])
