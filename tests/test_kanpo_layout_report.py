import hashlib
import json
from pathlib import Path
import sys
import unittest

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / 'tools'))

class KanpoReportTests(unittest.TestCase):
    def test_frozen_gold_and_source_are_unchanged(self):
        base = ROOT / 'tests/fixtures/kanpo_layout'
        frozen = json.loads((base / 'frozen-before-recognition.json').read_text())
        for name, expected in frozen['files'].items():
            self.assertEqual(hashlib.sha256((base / name).read_bytes()).hexdigest(), expected, name)
        confirmation = json.loads((base / 'confirmation-frozen-before-recognition.json').read_text())
        for name, expected in confirmation['files'].items():
            self.assertEqual(hashlib.sha256((base / name).read_bytes()).hexdigest(), expected, name)
        self.assertEqual((base / 'confirmation-two-columns.txt').read_text(), ''.join((base / f'confirmation-col{i}.txt').read_text() for i in (1,2)))
        final = json.loads((base / 'final-holdout-frozen-before-recognition.json').read_text())
        for name, expected in final['files'].items():
            self.assertEqual(hashlib.sha256((base / name).read_bytes()).hexdigest(), expected, name)
        self.assertEqual((base / 'final-holdout-two-columns.txt').read_text(), ''.join((base / f'final-holdout-col{i}.txt').read_text() for i in (1,2)))
        self.assertEqual((base / 'heldout-page.txt').read_text(), ''.join((base / f'heldout-col{i}.txt').read_text() for i in range(1,5)))

    def test_metric_preserves_repetitions_and_detects_excess(self):
        from summarize_kanpo_layout import score
        result = score('Ａ\n本文\n本文\n末尾', 'A\n本文\n本文\n本文\n末尾')
        self.assertEqual(result['edits'], 2)
        self.assertEqual(result['excess_exact_lines'], 1)
        self.assertEqual(result['characters'], 7)
        self.assertEqual(score('Ａ 本文', 'A本文')['edits'], 0)

    def test_edit_breakdown_distinguishes_deletions_insertions_and_substitutions(self):
        from summarize_kanpo_layout import score
        self.assertEqual(score('ABC', 'AC')['deletions'], 1)
        self.assertEqual(score('ABC', 'ABBC')['insertions'], 1)
        self.assertEqual(score('ABC', 'ADC')['substitutions'], 1)

    def test_notice_order_also_covers_non_bankruptcy_case_types(self):
        from summarize_kanpo_layout import score
        truth = '令和8年（再イ）第3号\n令和8年（再イ）第4号'
        self.assertFalse(score(truth, '令和8年（再イ）第4号\n令和8年（再イ）第3号')['notice_order_equal'])

    def test_missing_edges_and_number_order_are_visible(self):
        from summarize_kanpo_layout import score
        truth = '令和8年（フ）第3号\n本文\n令和8年（フ）第4号\n末尾'
        result = score(truth, '令和8年（フ）第4号\n本文\n令和8年（フ）第3号')
        self.assertFalse(result['notice_order_equal'])
        self.assertFalse(result['suffix_equal'])
        self.assertGreater(result['edits'], 0)

if __name__ == '__main__':
    unittest.main()
