import random
import unittest
from missing_line_context_probe import uncovered_ink_stats, comparison_metrics, candidate_strata

def image(width, height, points, tone=0):
    rgb = bytearray([255] * (width * height * 3))
    for (x, y) in points:
        rgb[(y * width + x) * 3:(y * width + x + 1) * 3] = bytes([tone] * 3)
    return bytes(rgb)

class UncoveredInkTests(unittest.TestCase):

    def test_union_is_half_open_clipped_and_never_double_counts(self):
        rgb = image(8, 4, [(0, 0), (2, 1), (3, 1), (4, 1), (6, 3), (7, 3)])
        result = uncovered_ink_stats(rgb, 8, 4, [-1, -2, 9, 5], [[0, 0, 3, 2], [1, 0, 3, 2], [6, 3, 8, 4], [0, 0, 3, 2]])
        self.assertEqual(result, {'foreground_pixels': 6, 'covered_pixels': 4, 'uncovered_pixels': 2, 'uncovered_columns': 2, 'uncovered_bbox': [3, 1, 5, 2]})
        self.assertEqual(uncovered_ink_stats(rgb, 8, 4, [3, 1, 4, 2], [])['uncovered_pixels'], 1)
        self.assertEqual(uncovered_ink_stats(rgb, 8, 4, [3, 1, 4, 2], [[3, 1, 4, 2]])['uncovered_pixels'], 0)

    def test_whitespace_threshold_and_invalid_input_are_explicit(self):
        for (tone, expected) in [(219, 1), (220, 0), (255, 0)]:
            result = uncovered_ink_stats(image(1, 1, [(0, 0)], tone), 1, 1, [0, 0, 1, 1], [])
            self.assertEqual(result['uncovered_pixels'], expected)
            self.assertEqual(result['uncovered_bbox'], [0, 0, 1, 1] if expected else None)
        for (rgb, w, h, bounds) in [(b'', 0, 1, [0, 0, 1, 1]), (b'xxx', 1, 2, [0, 0, 1, 1]), (b'xxx', 1, 1, [1, 0, 0, 1])]:
            with self.assertRaises(ValueError):
                uncovered_ink_stats(rgb, w, h, bounds, [])

    def test_partition_and_scale_translation_box_order_properties(self):
        rng = random.Random(113)
        for _ in range(80):
            (w, h) = (rng.randrange(2, 10), rng.randrange(2, 8))
            points = {(rng.randrange(w), rng.randrange(h)) for _ in range(14)}
            boxes = []
            for _ in range(4):
                (x0, x1) = sorted(rng.sample(range(w + 1), 2))
                (y0, y1) = sorted(rng.sample(range(h + 1), 2))
                boxes.append([x0, y0, x1, y1])
            base = uncovered_ink_stats(image(w, h, points), w, h, [0, 0, w, h], boxes)
            self.assertEqual(base['covered_pixels'] + base['uncovered_pixels'], len(points))
            (scale, dx, dy) = (rng.randrange(1, 4), rng.randrange(4), rng.randrange(4))
            points2 = {(x * scale + dx + i, y * scale + dy + j) for (x, y) in points for i in range(scale) for j in range(scale)}
            boxes2 = [[a * scale + dx, b * scale + dy, c * scale + dx, d * scale + dy] for (a, b, c, d) in reversed(boxes + boxes)]
            result = uncovered_ink_stats(image(w * scale + dx, h * scale + dy, points2), w * scale + dx, h * scale + dy, [dx, dy, w * scale + dx, h * scale + dy], boxes2)
            self.assertEqual(result['uncovered_pixels'], base['uncovered_pixels'] * scale * scale)
            self.assertEqual(result['uncovered_columns'], base['uncovered_columns'] * scale)
            expected = base['uncovered_bbox']
            if expected is not None:
                (a, b, c, d) = expected
                expected = [a * scale + dx, b * scale + dy, c * scale + dx, d * scale + dy]
            self.assertEqual(result['uncovered_bbox'], expected)

class ContextDistanceTests(unittest.TestCase):

    def test_width_sensitive_distances_and_literal_truth_are_kept(self):
        result = comparison_metrics('ＡＡ A', 'Ａ A', 'AA A', True)
        self.assertEqual(result['none']['before_char_errors'], 1)
        self.assertEqual(result['none']['candidate_char_errors'], 2)
        self.assertEqual(result['width']['candidate_char_errors'], 0)
        self.assertEqual(result['strict']['candidate_char_errors'], 2)
        self.assertEqual(result['none']['candidate_word_errors'], 1)

    def test_partial_crop_is_not_scored_as_full_line_truth(self):
        self.assertIsNone(comparison_metrics('ABCD', 'ABCD', 'BC', False))

class ContextStrataTests(unittest.TestCase):

    def test_relative_evidence_is_not_silently_lost_to_a_fixed_floor(self):
        self.assertEqual(candidate_strata(True, True, True, 121, 0.893, 0.737), ['relative_without_fixed_floor'])
        self.assertEqual(candidate_strata(True, True, True, 3, 0.943, 0.955), ['full_native_single_missing_high'])
        self.assertEqual(len(candidate_strata(True, True, True, 3, 0.96, 0.95)), 3)
        for score in [0, -1, 1.1, float('nan'), float('inf')]:
            self.assertEqual(candidate_strata(True, True, True, 3, score, 0.5), [])
        for (full, native, single, ink) in [(False, True, True, 2), (True, False, True, 2), (True, True, False, 2), (True, True, True, 0)]:
            self.assertEqual(candidate_strata(full, native, single, ink, 0.99, 0.5), [])
        self.assertEqual(candidate_strata(True, True, True, 2, 0.99, None), [])
if __name__ == '__main__':
    unittest.main()
