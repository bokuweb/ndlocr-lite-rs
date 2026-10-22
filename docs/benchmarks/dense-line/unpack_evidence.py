"""Verify every archive/member before extracting the frozen OCR evidence."""
from pathlib import Path, PurePosixPath
import argparse, hashlib, json, zipfile

def unpack(directory, stem, output):
    archive = directory / (stem + '.zip')
    metadata = json.loads((directory / (stem + '.json')).read_text())
    assert hashlib.sha256(archive.read_bytes()).hexdigest() == metadata['sha256']
    with zipfile.ZipFile(archive) as data:
        assert len(data.namelist()) == len(set(data.namelist()))
        assert set(data.namelist()) == set(metadata['members'])
        for member, digest in metadata['members'].items():
            part = PurePosixPath(member)
            assert not part.is_absolute() and '..' not in part.parts
            content = data.read(member)
            assert hashlib.sha256(content).hexdigest() == digest, member
            destination = output.joinpath(*part.parts)
            destination.parent.mkdir(parents=True, exist_ok=True)
            destination.write_bytes(content)
    print(stem, len(metadata['members']), 'verified members')

if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    directory = Path(__file__).resolve().parent
    unpack(directory, 'text-evidence', args.output)
    (args.output / 'evaluation-manifest.json').write_bytes((directory / 'evaluation-manifest.json').read_bytes())
