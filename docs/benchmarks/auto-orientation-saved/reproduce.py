#!/usr/bin/env python3
"""Verify saved directional NN pairs and reproduce scores without ONNX."""
from pathlib import Path
import argparse
import csv
import hashlib
import importlib.util
import json
import subprocess
import sys
import zipfile


def sha(b):
    return hashlib.sha256(b).hexdigest()


class Packet:
    def __init__(self, directory, expected):
        self.files, self.aliases = {}, {}
        for name, digest in expected.items():
            index = json.loads((directory / (name + '.json')).read_text())
            path = directory / (name + '.zip')
            assert sha(path.read_bytes()) == index['sha256'] == digest
            z = zipfile.ZipFile(path)
            assert set(z.namelist()) == set(index['members'])
            for member, h in index['members'].items():
                self.files[member] = (z, h)
            alias_name = name + '-aliases.json'
            self.aliases.update(json.loads(self.read(alias_name)))

    def read(self, path):
        if path in self.files:
            z, h = self.files[path]
            b = z.read(path)
        else:
            alias = self.aliases[path]
            b, h = self.read(alias['source']), alias['sha256']
        assert sha(b) == h, path
        return b


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, required=True)
    root = parser.parse_args().output.resolve()
    root.mkdir(parents=True)
    packet = Path(__file__).resolve().parent
    index = json.loads((packet / 'evidence.json').read_text())
    archive = packet / 'evidence.zip'
    assert sha(archive.read_bytes()) == index['sha256']
    with zipfile.ZipFile(archive) as z:
        assert set(z.namelist()) == set(index['members'])
        for name, h in index['members'].items():
            p = Path(name)
            assert not p.is_absolute() and '..' not in p.parts
            b = z.read(name)
            assert sha(b) == h
            target = root / p
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_bytes(b)
    evidence = root / 'evidence'
    expected = json.loads((evidence / 'result.json').read_text())
    assert sha((evidence / 'run.py').read_bytes()) == expected['driver_sha256']
    assert sha((evidence / 'audit.py').read_bytes()) == expected['pairing_sha256']
    assert sha((evidence / 'preregistered-spec.md').read_bytes()) == expected['spec_sha256']
    scorer = root / 'source/eval_ocr_metrics.py'
    assert sha(scorer.read_bytes()) == expected['scorer_sha256']
    sys.dont_write_bytecode = True
    spec = importlib.util.spec_from_file_location('verified_pairing', evidence / 'audit.py')
    helper = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(helper)
    parents = json.loads((packet / 'parents.json').read_text())
    packets = {name: Packet(packet.parent / name, archives) for name, archives in parents.items()}
    summary, pairs_count, rgb_count = [], 0, 0
    for name, parent, source_root, data_root, diagnostic in [
        ('bridge', 'single-pixel-body-bridge', 'tmp/closer-bridge', 'data', 'diagnostics'),
        ('detached', 'detached-body-components', 'tmp/detached-positive', 'positive', 'diagnostics/positive'),
    ]:
        source = packets[parent]
        def read(path, old):
            b = source.read(path)
            assert sha(b) == expected['sources_sha256'][source_root + '/' + old]
            return json.loads(b)
        am = read(data_root + '/fresh/evaluation-manifest.json', 'fresh/evaluation-manifest.json')
        hm = read(data_root + '/horizontal/evaluation-manifest.json', 'horizontal/evaluation-manifest.json')
        auto = [x for x in am['inputs'] if x['rotated_for_auto']]
        horizontal = hm['inputs']
        assert len(horizontal) == (128 if name == 'bridge' else 64)
        ids = {x['id'] for x in horizontal}
        truth = root / 'reproduced' / name / 'truth'
        truth.mkdir(parents=True)
        for x in horizontal:
            rgb = source.read(data_root + '/fresh/rgb/' + x['id'] + '.rgb')
            assert sha(rgb) == x['rgb_sha256']
            gt = source.read(data_root + '/fresh/truth/' + x['id'] + '.txt')
            assert sha(gt) == x['truth_sha256']
            assert gt == (evidence / name / 'truth' / (x['id'] + '.txt')).read_bytes()
            (truth / (x['id'] + '.txt')).write_bytes(gt)
            rgb_count += 1
        for model in ['30', '50', '100']:
            ar = read(diagnostic + '/fresh/' + model + '-export.json', 'fresh/' + model + '-export.json')
            hr = read(diagnostic + '/horizontal/' + model + '-export.json', 'horizontal/' + model + '-export.json')
            pairs = helper.paired_rows(auto, horizontal, [x for x in ar if x['id'] in ids], hr)
            pairs_count += len(pairs)
            base = root / 'reproduced' / name / model
            for phase in ['auto', 'horizontal']:
                (base / phase).mkdir(parents=True)
            for id, a, h in pairs:
                for phase, text in [('auto', a), ('horizontal', h)]:
                    b = text.encode()
                    assert b == (evidence / name / model / phase / (id + '.txt')).read_bytes()
                    (base / phase / (id + '.txt')).write_bytes(b)
            for norm in ['none', 'width', 'strict']:
                out = base / (norm + '.csv')
                subprocess.run([sys.executable, str(scorer), '--truth-dir', str(truth), '--system', 'auto=' + str(base / 'auto'), '--system', 'horizontal=' + str(base / 'horizontal'), '--normalize', norm, '--require-all', '--output-csv', str(out), '--output-md', str(out.with_suffix('.md'))], check=True, capture_output=True)
                for p in [out, out.with_suffix('.md')]:
                    assert p.read_bytes() == (evidence / name / model / p.name).read_bytes()
                rows = list(csv.DictReader(out.open()))
                a = {x['file']: x for x in rows if x['system'] == 'auto'}
                h = {x['file']: x for x in rows if x['system'] == 'horizontal'}
                s = dict(corpus=name, model=model, normalization=norm, images=len(pairs), changed=sum(x[1] != x[2] for x in pairs))
                for metric in ['char', 'word']:
                    b = [int(a[id][metric + '_errors']) for id in sorted(a)]
                    c = [int(h[id][metric + '_errors']) for id in sorted(a)]
                    s[metric] = {'before': sum(b), 'after': sum(c), 'improved': sum(y < x for x, y in zip(b, c)), 'regressed': sum(y > x for x, y in zip(b, c)), 'equal': sum(y == x for x, y in zip(b, c)), 'exact_recovered': sum(x > 0 and y == 0 for x, y in zip(b, c)), 'exact_lost': sum(x == 0 and y > 0 for x, y in zip(b, c))}
                summary.append(s)
    assert summary == expected['summary'] and (pairs_count, rgb_count) == (576, 192)
    result = {'evidence_sha256': index['sha256'], 'parents': parents, 'raw_pairs': pairs_count, 'source_rgb_and_truth_verified': rgb_count, 'table_files_byte_equal': 36, 'all_18_summaries_equal': True, 'new_nn_forwards': 0, 'strict_char_regressions': sum(x['char']['regressed'] for x in summary if x['normalization'] == 'strict')}
    (root / 'verification.json').write_text(json.dumps(result, indent=2) + '\n')
    print(json.dumps(result, indent=2))


if __name__ == '__main__':
    main()
