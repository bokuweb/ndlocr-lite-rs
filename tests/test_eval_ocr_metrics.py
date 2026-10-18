"""全角ASCIIの差に隠れるI/l退行を採点で検出する。"""
import importlib.util
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
import csv

SCRIPT = Path(__file__).resolve().parents[1] / 'tools/eval_ocr_metrics.py'
SPEC = importlib.util.spec_from_file_location('eval_ocr_metrics', SCRIPT)
metrics = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = metrics
SPEC.loader.exec_module(metrics)


class WidthEvaluationTests(unittest.TestCase):
    """写像範囲と従来指標の互換性を固定する。"""

    def test_width_reveals_ai_regression_hidden_by_raw_distance(self):
        truth, original, candidate = 'ＡＩ関連企', 'AI関連企', 'Al関連企'
        self.assertEqual(metrics.levenshtein(list(truth), list(original)), 2)
        self.assertEqual(metrics.levenshtein(list(truth), list(candidate)), 2)
        fold = lambda text: list(metrics.normalize_text(text, 'width'))
        self.assertEqual(metrics.levenshtein(fold(truth), fold(original)), 0)
        self.assertEqual(metrics.levenshtein(fold(truth), fold(candidate)), 1)

    def test_exactly_the_fullwidth_ascii_range_is_folded(self):
        self.assertEqual(metrics.normalize_text(''.join(map(chr, range(0xFF01, 0xFF5F))), 'width'),
                         ''.join(map(chr, range(0x21, 0x7F))))

    def test_width_retains_other_compatibility_forms_and_whitespace(self):
        text = '\r\n  ＡaＡ\t\u3000①㍿ﬃｶﾞガ\uFF00\uFF5F\n '
        self.assertEqual(metrics.normalize_text(text, 'width'),
                         '\r\n  AaA\t\u3000①㍿ﬃｶﾞガ\uFF00\uFF5F\n ')

    def test_existing_modes_keep_their_behavior(self):
        text = '  Ａ  A.\r\nB!  '
        self.assertEqual(metrics.normalize_text(text, 'none'), text)
        self.assertEqual(metrics.normalize_text(text, 'basic'), 'Ａ A.\nB!')
        self.assertEqual(metrics.normalize_text(text, 'strict'), 'ＡA\nB')

    def test_cli_width_mode_writes_the_semantic_substitution_to_csv(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            for name, text in [('truth', 'ＡＩ関連企'), ('original', 'AI関連企'), ('candidate', 'Al関連企')]:
                (root / name).mkdir()
                (root / name / 'case.txt').write_text(text, encoding='utf-8')
            output = root / 'metrics.csv'
            result = subprocess.run([sys.executable, str(SCRIPT), '--truth-dir', str(root / 'truth'),
                '--system', f'original={root / "original"}', '--system', f'candidate={root / "candidate"}',
                '--normalize', 'width', '--require-all', '--output-csv', str(output)],
                capture_output=True, text=True)
            self.assertEqual(result.returncode, 0, result.stderr)
            with output.open(encoding='utf-8', newline='') as file:
                rows = list(csv.DictReader(file))
            self.assertEqual([(row['system'], row['char_errors'], row['char_total']) for row in rows],
                             [('original', '0', '5'), ('candidate', '1', '5')])


if __name__ == '__main__':
    unittest.main()
