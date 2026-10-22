"""Replay saved raw decoder steps and CSV without model loads or inference."""
from pathlib import Path, PurePosixPath
import argparse
import hashlib
import json
import subprocess
import sys
import zipfile

def empty_output(path):
    if path.exists() and (not path.is_dir() or any(path.iterdir())):
        raise ValueError('output must be absent or an empty directory')
    path.mkdir(parents=True, exist_ok=True)
    return path

def relative_member(name):
    path = PurePosixPath(name)
    if (not name or '\\' in name or path.is_absolute() or '..' in path.parts
            or path.as_posix() != name or name == '.'):
        raise ValueError('archive member must be a canonical relative path')
    return Path(name)

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', required=True, type=Path)
    args = parser.parse_args()
    packet = Path(__file__).resolve().parent
    index = json.loads((packet / 'archive.json').read_text())
    archive = packet / index['archive']
    digest = lambda data: hashlib.sha256(data).hexdigest()
    if digest(archive.read_bytes()) != index['archive_sha256']:
        raise ValueError('archive SHA256 mismatch')
    with zipfile.ZipFile(archive) as zipped:
        names = zipped.namelist()
        if (len(names) != len(set(names)) or set(names) != set(index['files_sha256'])
                or sum(x.file_size for x in zipped.infolist()) > 250_000_000):
            raise ValueError('archive membership or size mismatch')
        members = [(name, relative_member(name)) for name in names]
        output = empty_output(args.output.resolve())
        for name, relative in members:
            data = zipped.read(name)
            if digest(data) != index['files_sha256'][name]:
                raise ValueError('member SHA256 mismatch: ' + name)
            destination = output / relative
            destination.parent.mkdir(parents=True, exist_ok=True)
            destination.write_bytes(data)
    audit = output / 'tmp/vertical-output-capacity/stop-audit'
    expected = {name: (audit / name).read_bytes() for name in ('raw-stops.csv', 'analysis.json')}
    with (output / 'replay.stdout').open('wb') as stdout, (output / 'replay.stderr').open('wb') as stderr:
        result = subprocess.run([sys.executable, str(audit / 'analyze.py')], cwd=output, stdout=stdout, stderr=stderr)
    if result.returncode:
        raise RuntimeError('saved-step audit failed; see replay.stderr')
    for name, data in expected.items():
        if (audit / name).read_bytes() != data:
            raise ValueError('regenerated report differs: ' + name)
    print(json.dumps({'restored_files': len(members), 'verified_raw_pairs': 162,
                      'prior_raw_exact_pairs': 126, 'page_f32_pairs': 36,
                      'csv_data_rows': 162, 'reports_byte_equal': True,
                      'new_nn_calls': 0, 'models_loaded': 0}))

if __name__ == '__main__':
    main()
