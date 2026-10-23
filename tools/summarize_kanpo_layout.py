#!/usr/bin/env python3
"""Re-score frozen kanpo benchmark reports, checking every raw prediction hash."""
import argparse
from collections import Counter
import hashlib
import json
from pathlib import Path
import re
import statistics
import unicodedata

from eval_ocr_metrics import levenshtein


def normalize(text):
    return ''.join(unicodedata.normalize('NFKC', text).split())


def edit_breakdown(a, b):
    # A minimal alignment; diagonal, deletion, insertion is the fixed tie priority.
    width = len(b) + 1
    steps = bytearray((len(a) + 1) * width)
    previous = list(range(width))
    for j in range(1, width):
        steps[j] = 2
    for i, ca in enumerate(a, 1):
        row = [i]
        steps[i * width] = 1
        for j, cb in enumerate(b, 1):
            diagonal = previous[j - 1] + (ca != cb)
            deletion = previous[j] + 1
            insertion = row[j - 1] + 1
            best = min(diagonal, deletion, insertion)
            row.append(best)
            steps[i * width + j] = 0 if best == diagonal else 1 if best == deletion else 2
        previous = row
    i, j = len(a), len(b)
    counts = {'substitutions': 0, 'deletions': 0, 'insertions': 0}
    while i or j:
        step = steps[i * width + j]
        if step == 0:
            counts['substitutions'] += a[i - 1] != b[j - 1]
            i -= 1
            j -= 1
        elif step == 1:
            counts['deletions'] += 1
            i -= 1
        else:
            counts['insertions'] += 1
            j -= 1
    return counts


def score(truth, prediction):
    a, b = normalize(truth), normalize(prediction)
    notices = lambda text: re.findall(r'令和\d+年\([^()]+\)第\d+号', text)
    expected = Counter(filter(None, map(normalize, truth.splitlines())))
    actual = Counter(filter(None, map(normalize, prediction.splitlines())))
    errors = levenshtein(list(a), list(b))
    breakdown = edit_breakdown(a, b)
    assert sum(breakdown.values()) == errors
    return {**breakdown, 'characters': len(a), 'prediction_characters': len(b),
            'edits': errors, 'cer': errors / len(a) if a else None,
            'excess_exact_lines': sum(max(0, actual[line] - count) for line, count in expected.items()),
            'prefix_equal': b[:24] == a[:24], 'suffix_equal': b[-24:] == a[-24:],
            'notice_order_equal': notices(a) == notices(b),
            'expected_notice_sequence': notices(a), 'actual_notice_sequence': notices(b)}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--manifest', type=Path, required=True)
    parser.add_argument('--run', action='append', required=True, help='label=benchmark directory')
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    manifest = json.loads(args.manifest.read_text())
    samples = {row['id']: row for row in manifest['samples']}
    for sample in samples.values():
        assert hashlib.sha256(Path(sample['image']).read_bytes()).hexdigest() == sample['sha256']
    report = {'scope': manifest['scope'], 'normalization': 'NFKC + remove whitespace, scoring only', 'runs': {}}
    cache = {}
    for spec in args.run:
        name, directory = spec.split('=', 1)
        directory = Path(directory)
        source = json.loads((directory / 'report.json').read_text())
        assert source['manifest'] == manifest, f'{name}: changed manifest'
        assert len(source['results']) == len(samples) * source['repeats'] * len({r['engine'] for r in source['results']})
        rows = []
        for row in source['results']:
            assert row['exit'] == 0
            path = directory / f"{row['sample']}-{row['engine']}-{row['repeat']}" / 'recognized.txt'
            raw = path.read_bytes()
            assert hashlib.sha256(raw).hexdigest() == row['prediction_sha256'], str(path)
            result = {k: row[k] for k in ['sample', 'engine', 'repeat', 'seconds', 'prediction_sha256']}
            result['peak_rss_bytes'] = row.get('peak_rss_bytes_time_wrapper')
            sample = samples[row['sample']]
            if 'truth' in sample:
                key = (sample['truth'], row['prediction_sha256'])
                if key not in cache:
                    cache[key] = score(sample['truth'], raw.decode())
                result.update(cache[key])
                assert result['edits'] == row['edits']
                assert result['characters'] == row['characters']
            rows.append(result)
        summary = []
        for sample in samples:
            for engine in sorted({r['engine'] for r in rows}):
                group = [r for r in rows if r['sample'] == sample and r['engine'] == engine]
                assert len(group) == source['repeats']
                assert sorted(r['repeat'] for r in group) == list(range(source['repeats']))
                summary.append({'sample': sample, 'engine': engine,
                                'median_seconds': statistics.median(r['seconds'] for r in group),
                                'median_peak_rss_bytes': statistics.median(r['peak_rss_bytes'] for r in group if r['peak_rss_bytes'] is not None) if any(r['peak_rss_bytes'] is not None for r in group) else None,
                                'deterministic': len({r['prediction_sha256'] for r in group}) == 1,
                                **{k: group[0][k] for k in ['characters', 'edits', 'cer', 'deletions', 'insertions', 'substitutions', 'excess_exact_lines', 'prefix_equal', 'suffix_equal', 'notice_order_equal'] if k in group[0]}})
        report['runs'][name] = {'rows': rows, 'summary': summary}
    args.output.write_text(json.dumps(report, ensure_ascii=False, indent=2) + '\n')
    for name, run in report['runs'].items():
        print(name)
        for row in run['summary']:
            print(json.dumps(row, ensure_ascii=False))


if __name__ == '__main__':
    main()
