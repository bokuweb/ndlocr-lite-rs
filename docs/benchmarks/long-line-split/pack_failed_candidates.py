"""Keep all rejected-candidate bytes in verified deterministic ZIP archives."""
from pathlib import Path
import hashlib, json, zipfile
root = Path('docs/benchmarks/long-line-split')
for phase in ['initial', 'refined-before-dedup']:
    directory = root / phase
    files = sorted(p for p in directory.rglob('*') if p.is_file() and p.name not in ['evidence.zip', 'archive.json'])
    archive = directory / 'evidence.zip'
    if not files and archive.exists():
        index = json.loads((directory / 'archive.json').read_text())
        assert hashlib.sha256(archive.read_bytes()).hexdigest() == index['sha256']
        with zipfile.ZipFile(archive) as stream:
            assert set(stream.namelist()) == set(index['member_sha256'])
            for name, digest in index['member_sha256'].items():
                assert hashlib.sha256(stream.read(name)).hexdigest() == digest
        continue
    if not files:
        raise RuntimeError(f'no source reports or existing archive in {directory}')
    members = {p.relative_to(directory).as_posix(): hashlib.sha256(p.read_bytes()).hexdigest() for p in files}
    with zipfile.ZipFile(archive, 'w', compression=zipfile.ZIP_DEFLATED, compresslevel=9) as stream:
        for path in files:
            info = zipfile.ZipInfo(path.relative_to(directory).as_posix(), date_time=(1980,1,1,0,0,0))
            info.compress_type = zipfile.ZIP_DEFLATED
            info.external_attr = 0o100644 << 16
            stream.writestr(info, path.read_bytes())
    with zipfile.ZipFile(archive) as stream:
        assert set(stream.namelist()) == set(members)
        for name, digest in members.items():
            assert hashlib.sha256(stream.read(name)).hexdigest() == digest
    (directory / 'archive.json').write_text(json.dumps({'archive': 'evidence.zip', 'sha256': hashlib.sha256(archive.read_bytes()).hexdigest(), 'member_sha256': members}, indent=2) + '\n')
