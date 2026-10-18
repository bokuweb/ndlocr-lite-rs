"""実画像を読む前に候補評価の定義を固定する回帰。"""
import importlib.util
from pathlib import Path
import unittest

spec = importlib.util.spec_from_file_location('coverage', Path(__file__).parents[1] / 'tools/eval_candidate_coverage.py')
coverage = importlib.util.module_from_spec(spec)
spec.loader.exec_module(coverage)


def row(text, candidates=None):
    """原文と同じ順序の評価用文字位置を作る。"""
    return {'id': 'one', 'text': text, 'positions': [
        {'character': c, 'candidates': cs} for c, cs in zip(text, candidates or [[c] for c in text])
    ]}


class CandidateCoverageTests(unittest.TestCase):
    """文字を補正せず、候補に残る範囲と編集の種類を分ける。"""

    def test_substitution_missing_extra_and_eos_rank(self):
        actual = coverage.evaluate('あ１', row('あ1', [['あ','い'], ['1',None,'１']]), 3)
        self.assertEqual((actual['errors'], actual['matches'], actual['substitutions']), (1,1,1))
        self.assertEqual(actual['substitution_ranks'], {'3':1})
        self.assertEqual(actual['oracle_errors'], 0)
        self.assertEqual(actual['correct_with_alternatives'], 1)
        self.assertEqual(coverage.evaluate('あ１', row('あ'), 3)['deletions'], 1)
        self.assertEqual(coverage.evaluate('あ', row('あ１'), 3)['insertions'], 1)

    def test_strict_names_digits_repeats_spaces_and_width(self):
        for text in ['テスト株式会社', '111111', '１ 1', 'ruri', '']:
            self.assertEqual(coverage.evaluate(text, row(text), 1)['errors'], 0)
        self.assertEqual(coverage.evaluate('１', row('1'), 10)['errors'], 1)
        self.assertEqual(coverage.evaluate('a a', row('aa'), 10)['oracle_errors'], 1)

    def test_fixed_alignment_and_oracle_are_distinct(self):
        actual = coverage.evaluate('ab', row('ba', [['b','a'], ['a','b']]), 2)
        self.assertEqual(actual['substitutions'], 2)
        self.assertEqual(actual['oracle_errors'], 0)
        self.assertEqual(actual['substitution_ranks'], {'2':2})
        self.assertEqual(coverage.evaluate('ab', row('a'), 2)['oracle_errors'], 1)

    def test_more_candidates_never_worsen_oracle(self):
        import itertools
        for truth in map(''.join, itertools.product('あ１', repeat=3)):
            candidate = row('１あ１', [['１',None,'あ'], ['あ','１'], ['１','あ']])
            scores = [coverage.evaluate(truth, candidate, k)['oracle_errors'] for k in range(1,5)]
            self.assertEqual(scores, sorted(scores, reverse=True))
            self.assertEqual(scores[0], coverage.evaluate(truth, candidate, 1)['errors'])

    def test_invalid_mapping_rejected(self):
        for bad in [row('a', [['b']]), row('a', [[]]), {'id':'one','text':'ab','positions':[]}]:
            with self.assertRaises(ValueError):
                coverage.evaluate('a', bad, 1)
        with self.assertRaises(ValueError):
            coverage.evaluate('a', row('a'), 0)

    def test_manifest_ids_are_exact_and_unique(self):
        truth = [{'id':'one','truth':'１'}]
        self.assertEqual(coverage.evaluate_manifest(truth, [row('１')], [1])['by_k']['1']['errors'], 0)
        for bad in [[], [row('１'),row('１')], [dict(row('１'), id='extra')]]:
            with self.assertRaises(ValueError):
                coverage.evaluate_manifest(truth, bad, [1])
        with self.assertRaises(ValueError):
            coverage.evaluate_manifest(truth*2, [row('１')], [1])
