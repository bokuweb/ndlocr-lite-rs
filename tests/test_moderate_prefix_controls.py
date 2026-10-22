from collections import Counter
from pathlib import Path
import sys
import unittest
from PIL import ImageFont
sys.path.insert(0,str(Path(__file__).resolve().parents[1]/'tools'))


class ModeratePrefixControlsTest(unittest.TestCase):
    def test_plan_covers_all_fixed_conditions_without_duplicate_ids(self):
        from moderate_prefix_controls import control_plan
        plan=control_plan(['arial','bold','serif'])
        self.assertEqual(len(plan),72)
        self.assertEqual(len({r['id'] for r in plan}),72)
        self.assertEqual(Counter(r['cohort'] for r in plan),{'body':36,'repeated':12,'portrait':12,'nonletter':12})
        self.assertEqual({len(r['text']) for r in plan if r['cohort']=='body'},{67,87,107,127,147,167})
        self.assertTrue(all(not r['text'].isspace() for r in plan if r['cohort']!='nonletter'))

    def test_missing_duplicate_or_changed_font_labels_cannot_silently_reduce_the_scope(self):
        from moderate_prefix_controls import control_plan
        for labels in [[],['arial','bold'],['arial','arial','serif'],['arial','bold','unknown']]:
            with self.subTest(labels=labels),self.assertRaises(ValueError):control_plan(labels)

    def test_missing_glyph_is_rejected_instead_of_becoming_a_labeled_tofu_image(self):
        from moderate_prefix_controls import assert_font_covers
        font=ImageFont.load_default()
        assert_font_covers(font,'No.12')
        with self.assertRaises(ValueError):assert_font_covers(font,'港')
