from pathlib import Path
import sys
import unittest
from PIL import Image
sys.path.insert(0,str(Path(__file__).resolve().parents[1]/'tools'))


class ModeratePrefixPacketTest(unittest.TestCase):
    def test_all_frozen_controls_and_crops_can_be_replayed_without_models_or_fonts(self):
        from verify_moderate_prefix_packet import verify
        result=verify(Path(__file__).resolve().parents[1]/'docs/benchmarks/vertical-moderate-prefix')
        self.assertEqual(result,{'physical_inputs':72,'geometry_inputs':216,'part_RGBs':2952,'independent_pixels':186559200,'glyph_cut_events':0,'NN_calls':0,'R_policy_started':False})

    def test_route_rejects_gap_overlap_wrong_pixels_and_glyph_interior_cuts(self):
        from verify_moderate_prefix_packet import verify_route
        image=Image.new('RGB',(2,6))
        image.putdata([(x*17,y*23,99) for y in range(6) for x in range(2)])
        rotated=image.transpose(Image.Transpose.ROTATE_90)
        def parts(ranges):
            return [{'file':str(i),'range':[a,b],'width':b-a,'height':2,'space_before':False} for i,(a,b) in enumerate(ranges)]
        valid=parts([(0,3),(3,6)])
        raw={p['file']:rotated.crop((p['range'][0],0,p['range'][1],2)).tobytes() for p in valid}
        self.assertEqual(verify_route(image,valid,raw.__getitem__,[[0,0,2,2],[0,4,2,6]]),12)
        for ranges in [[(0,2),(3,6)],[(0,4),(3,6)],[(0,3),(3,5)],[(0,0),(0,6)]]:
            with self.subTest(ranges=ranges),self.assertRaises(AssertionError):verify_route(image,parts(ranges),raw.__getitem__,[])
        with self.assertRaises(AssertionError):verify_route(image,valid,lambda name:b'\0'*len(raw[name]),[])
        with self.assertRaises(AssertionError):verify_route(image,valid,raw.__getitem__,[[0,2,2,4]])
