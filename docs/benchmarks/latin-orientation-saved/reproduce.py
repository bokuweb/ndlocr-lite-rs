#!/usr/bin/env python3
"""Verify saved RGB/raw pairs and reproduce directional scores without ONNX."""
from pathlib import Path
import argparse
import hashlib
import json
import shutil
import subprocess
import sys
import zipfile


def sha(data):
    return hashlib.sha256(data).hexdigest()


class Packet:
    def __init__(self, directory, expected):
        self.files, self.aliases = {}, {}
        for name, digest in expected.items():
            index = json.loads((directory / (name + '.json')).read_text())
            archive = directory / (name + '.zip')
            assert sha(archive.read_bytes()) == index['sha256'] == digest
            z = zipfile.ZipFile(archive)
            assert set(z.namelist()) == set(index['members'])
            for member, h in index['members'].items():
                assert member not in self.files
                self.files[member] = (z, h)
            aliases = json.loads(z.read(name + '-aliases.json'))
            for key, alias in aliases.items():
                assert alias['source'] in index['members']
                assert alias['sha256'] == index['members'][alias['source']]
                assert key not in self.aliases and key not in index['members']
                self.aliases[key] = alias

    def read(self, name):
        if name in self.files:
            z, digest = self.files[name]
            data = z.read(name)
        else:
            alias = self.aliases[name]
            data, digest = self.read(alias['source']), alias['sha256']
        assert sha(data) == digest, name
        return data


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
        for name, digest in index['members'].items():
            relative = Path(name)
            assert not relative.is_absolute() and '..' not in relative.parts
            data = z.read(name)
            assert sha(data) == digest
            target = root / relative
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_bytes(data)
    evidence = root / 'evidence'
    expected = json.loads((evidence / 'result.json').read_text())
    for name, field in [('run.py', 'driver_sha256'), ('audit.py', 'pairing_sha256'),
                        ('preregistered-spec.md', 'spec_sha256')]:
        assert sha((evidence / name).read_bytes()) == expected[field]
    assert sha((root / 'source/eval_ocr_metrics.py').read_bytes()) == expected['scorer_sha256']
    parents = json.loads((packet / 'parents.json').read_text())
    source = Packet(packet.parent / parents['packet'], parents['archives'])
    old_root = 'tmp/latin-pair-policy/'
    for path, digest in expected['sources_sha256'].items():
        assert path.startswith(old_root)
        data = source.read('diagnostics/' + path[len(old_root):])
        assert sha(data) == digest
        target = root / path
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_bytes(data)
    am = json.loads((root / old_root / 'fresh/evaluation-manifest.json').read_text())
    hm = json.loads((root / old_root / 'horizontal/evaluation-manifest.json').read_text())
    auto = [x for x in am['inputs'] if x['rotated_for_auto']]
    assert len(auto) == len(hm['inputs']) == 636
    for row in auto + hm['inputs']:
        for field in ['rgb', 'effective_rgb', 'image', 'original', 'truth']:
            path = row[field]
            assert path.startswith(old_root)
            if field == 'truth':
                data = (evidence / 'truth' / (row['id'] + '.txt')).read_bytes()
            else:
                data = source.read('data/' + path[len(old_root):])
            assert sha(data) == row[field + '_sha256']
            target = root / path
            target.parent.mkdir(parents=True, exist_ok=True)
            if target.exists():
                assert target.read_bytes() == data
            else:
                target.write_bytes(data)
    work = root / 'tmp/latin-orientation-saved'
    work.mkdir(parents=True)
    for name in ['run.py', 'audit.py', 'test_pairs.py', 'test_pixels.py']:
        shutil.copyfile(evidence / name, work / name)
    spec = root / 'docs/latin_auto_orientation_saved_audit.md'
    spec.parent.mkdir(parents=True)
    shutil.copyfile(evidence / 'preregistered-spec.md', spec)
    tools = root / 'tools'
    tools.mkdir()
    shutil.copyfile(root / 'source/eval_ocr_metrics.py', tools / 'eval_ocr_metrics.py')
    commands = [[sys.executable, '-m', 'unittest', 'discover', '-s', str(work), '-p', 'test_*.py'],
                [sys.executable, str(work / 'run.py')]]
    for number, argv in enumerate(commands):
        with (root / f'command-{number}.stdout').open('wb') as stdout, (root / f'command-{number}.stderr').open('wb') as stderr:
            subprocess.run(argv, cwd=root, stdout=stdout, stderr=stderr, check=True)
    assert (work / 'result.json').read_bytes() == (evidence / 'result.json').read_bytes()
    count = 0
    for model in ['30', '50', '100']:
        for norm in ['none', 'width', 'strict']:
            for ext in ['csv', 'md']:
                path = Path(model) / (norm + '.' + ext)
                assert (work / path).read_bytes() == (evidence / path).read_bytes()
                count += 1
    result = dict(evidence_sha256=index['sha256'], parent_archives=parents['archives'],
                  raw_pairs=1908, original_images=636, table_files_byte_equal=count,
                  recomputed_summary_and_regressions_equal=True, new_nn_forwards=0)
    (root / 'verification.json').write_text(json.dumps(result, indent=2) + '\n')
    print(json.dumps(result, indent=2))


if __name__ == '__main__':
    main()
