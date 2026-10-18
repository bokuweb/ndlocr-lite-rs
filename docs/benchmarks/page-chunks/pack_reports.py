"""Compress complete raw reports without dropping any records; keep the PR diff readable."""
from pathlib import Path
import gzip, hashlib, io, json
source = Path('tmp/page-chunk-probe')
destination = Path('docs/benchmarks/page-chunks')
files = {
    'fine-initial-results.json': source / 'results.json',
    'fine-results.json': source / 'results-enriched.json',
    'native-results.json': source / 'native/results.json',
    'raw-baseline-provenance.json': source / 'raw-baseline-provenance.json',
}
records = []
for name, path in files.items():
    data = path.read_bytes()
    buffer = io.BytesIO()
    with gzip.GzipFile(filename='', mode='wb', fileobj=buffer, compresslevel=9, mtime=0) as stream:
        stream.write(data)
    compressed = buffer.getvalue()
    assert gzip.decompress(compressed) == data
    (destination / (name + '.gz')).write_bytes(compressed)
    records.append({'file': name + '.gz', 'compression': 'gzip', 'bytes': len(data),
                    'sha256': hashlib.sha256(data).hexdigest(),
                    'compressed_sha256': hashlib.sha256(compressed).hexdigest()})
(destination / 'compressed-reports.json').write_text(json.dumps(records, indent=2) + '\n')
