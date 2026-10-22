"""Verify every archived byte and restore reproducible scoring inputs."""
from pathlib import Path,PurePosixPath
import argparse,hashlib,json,zipfile
parser=argparse.ArgumentParser();parser.add_argument('--output',type=Path,default=Path('tmp/page-band-fallback'));args=parser.parse_args()
root=Path(__file__).resolve().parent;meta=json.loads((root/'text-evidence.json').read_text(encoding="utf-8"));payload=(root/'text-evidence.zip').read_bytes();assert hashlib.sha256(payload).hexdigest()==meta['sha256']
with zipfile.ZipFile(root/'text-evidence.zip')as archive:
 assert set(archive.namelist())==set(meta['members'])
 for name,digest in meta['members'].items():
  part=PurePosixPath(name);assert not part.is_absolute()and'..'not in part.parts and part.parts[0]in['before','after','truth']
  data=archive.read(name);assert hashlib.sha256(data).hexdigest()==digest
  target=args.output.joinpath(*part.parts);target.parent.mkdir(parents=True,exist_ok=True);target.write_bytes(data)
manifest=json.loads((root/'evaluation-manifest.json').read_text(encoding="utf-8"));assert all(meta['members'][name]==digest for name,digest in manifest['prediction_sha256'].items())
(args.output/'evaluation-manifest.json').write_bytes((root/'evaluation-manifest.json').read_bytes());print('verified and restored',len(meta['members']),'files')
