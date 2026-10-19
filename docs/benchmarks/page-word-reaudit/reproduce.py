#!/usr/bin/env python3
"""Verify evidence and reproduce 78 official metric tables and two audits (stdlib only)."""
from pathlib import Path
import argparse, csv, hashlib, json, subprocess, sys, zipfile


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('destination', type=Path, help='New directory for verified evidence')
    root = parser.parse_args().destination.resolve()
    root.mkdir(parents=True, exist_ok=False)
    here = Path(__file__).resolve().parent
    sha = lambda data: hashlib.sha256(data).hexdigest()
    members = 0
    for archive in ['data', 'diagnostics']:
        path = here / (archive + '.zip')
        index = json.loads((here / (archive + '.json')).read_text(encoding='utf-8'))
        assert sha(path.read_bytes()) == index['sha256'], path
        with zipfile.ZipFile(path) as z:
            assert len(z.namelist()) == len(set(z.namelist())) == index['member_count']
            assert set(z.namelist()) == set(index['members'])
            for key, expected in index['members'].items():
                relative = Path(key)
                assert not relative.is_absolute() and '..' not in relative.parts, key
                data = z.read(key)
                assert sha(data) == expected, key
                target = root / relative
                target.parent.mkdir(parents=True, exist_ok=True)
                assert not target.exists(), key
                target.write_bytes(data)
                members += 1
    manifest = json.loads((root / 'final/evaluation-manifest.json').read_text(encoding='utf-8'))
    for row in manifest['inputs']:
        for kind in ['image', 'truth']:
            path = root / ('images' if kind == 'image' else 'truth') / (row['id'] + (Path(row[kind]).suffix if kind == 'image' else '.txt'))
            assert sha(path.read_bytes()) == row[kind + '_sha256'], path
    modes = ['deim', 'bands', 'runtime30_eval', 'runtime30_default', 'runtime100_eval', 'runtime100_default']
    scorer = root / 'candidate/source/tools/eval_ocr_metrics.py'
    tables = 0
    for phase, old, new in [('reaudit-metrics', 'previous_page', 'current'), ('metrics', 'current', 'candidate')]:
        output = root / 'reproduced' / phase
        output.mkdir(parents=True)
        changes, regressions, word_regressions, summary = [], [], [], []
        comparisons = 0
        for mode in modes:
            before, after = root / old / mode, root / new / mode
            for norm in ['none', 'width', 'strict']:
                stem = mode + '-' + norm
                table, report = output / (stem + '.csv'), output / (stem + '.md')
                subprocess.run([sys.executable, str(scorer), '--truth-dir', str(root / 'truth'), '--system', old + '=' + str(before), '--system', new + '=' + str(after), '--normalize', norm, '--require-all', '--output-csv', str(table), '--output-md', str(report)], check=True, capture_output=True)
                table.write_bytes(table.read_bytes().replace(b'\r\n', b'\n'))
                report.write_bytes((report.read_text(encoding='utf-8').rstrip() + '\n').encode('utf-8'))
                for path in [table, report]:
                    assert path.read_bytes() == (root / 'expected' / phase / path.name).read_bytes(), path
                    tables += 1
                records = list(csv.DictReader(table.open(encoding='utf-8', newline='')))
                b = {x['file']: x for x in records if x['system'] == old}
                a = {x['file']: x for x in records if x['system'] == new}
                assert len(a) == len(b) == 287
                stat = {'mode': mode, 'normalization': norm, 'images': 287, 'char_improved': 0, 'word_improved': 0, 'byte_same': 0, 'changed_char_ties': 0}
                for name in a:
                    bc, ac = int(b[name]['char_errors']), int(a[name]['char_errors'])
                    bw, aw = int(b[name]['word_errors']), int(a[name]['word_errors'])
                    same = (before / name).read_bytes() == (after / name).read_bytes()
                    comparisons += 1
                    if not same:
                        changes.append([mode, norm, name, bc, ac, bw, aw])
                    if ac > bc:
                        regressions.append([mode, norm, name, bc, ac])
                    if aw > bw:
                        word_regressions.append([mode, norm, name, bw, aw])
                    stat['char_improved'] += ac < bc
                    stat['word_improved'] += aw < bw
                    stat['byte_same'] += same
                    stat['changed_char_ties'] += not same and ac == bc
                summary.append(stat)
        audit = {'comparisons': comparisons, 'changes': changes, 'char_regressions': regressions, 'word_regressions': word_regressions, 'summary': summary}
        path = output / 'audit.json'
        path.write_bytes((json.dumps(audit, ensure_ascii=False, indent=2) + '\n').encode('utf-8'))
        assert path.read_bytes() == (root / 'expected' / phase / path.name).read_bytes(), path
        print(phase, comparisons, 'character and word comparisons reproduced')
    padding = root / 'work/raw-space-padding'
    output = root / 'reproduced/raw-space-padding'
    output.mkdir()
    for norm in ['none', 'width', 'strict']:
        table, report = output / (norm + '.csv'), output / (norm + '.md')
        command = [sys.executable, str(scorer), '--truth-dir', str(padding / 'truth'), '--normalize', norm, '--require-all', '--output-csv', str(table), '--output-md', str(report)]
        for model in ['30', '50', '100']:
            command += ['--system', model + '=' + str(padding / ('model' + model))]
        subprocess.run(command, check=True, capture_output=True)
        table.write_bytes(table.read_bytes().replace(b'\r\n', b'\n'))
        report.write_bytes(report.read_bytes().replace(b'\r\n', b'\n'))
        for path in [table, report]:
            assert path.read_bytes() == (padding / path.name).read_bytes(), path
            tables += 1
    print(members, 'verified members;', tables, 'byte-identical tables and two audits')


if __name__ == '__main__':
    main()
