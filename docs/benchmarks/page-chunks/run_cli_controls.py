from pathlib import Path
import concurrent.futures,hashlib,json,os,subprocess,time
root=Path('tmp/page-chunk-probe');binary=Path('tmp/url-hyphen/candidate-ndlocr').resolve()
images=sorted(Path('tests/fixtures/eval/images').glob('*.png'))+sorted(Path('docs/benchmarks/deim-edge/quality/images').glob('*'));assert len(images)==20
truth=root/'truth';truth.mkdir(exist_ok=True)
for image in images:
 source=image.stem.removesuffix('_jpeg90').removesuffix('_jpeg75').removesuffix('_half')
 (truth/(image.stem+'.txt')).write_bytes((Path('tests/fixtures/eval/truth')/(source+'.txt')).read_bytes())
manifest={'baseline_commit':'dde5d3a05e71f479fe3545869bed87560d962bd8','binary':{'path':str(binary),'sha256':hashlib.sha256(binary.read_bytes()).hexdigest()},'inputs':{str(p):hashlib.sha256(p.read_bytes()).hexdigest() for p in images},'truth':{p.name:hashlib.sha256(p.read_bytes()).hexdigest() for p in truth.glob('*.txt')},'model_sha256':{str(p):hashlib.sha256(p.read_bytes()).hexdigest() for p in Path('models').glob('*.onnx')},'charset_sha256':hashlib.sha256(Path('models/NDLmoji.yaml').read_bytes()).hexdigest()}
(root/'manifest.json').write_text(json.dumps(manifest,indent=2)+'\n')
def run(job):
 image,mode=job;folder=root/'cli'/mode;folder.mkdir(parents=True,exist_ok=True)
 argv=[str(binary),'recognize-page','--image',str(image),'--charset','models/NDLmoji.yaml','--enable-cascade','false','--split-long-lines','false','--use-deim-detection','true' if mode=='deim' else 'false','--output-txt',str(folder/(image.stem+'.txt'))]
 start=time.monotonic();p=subprocess.run(argv,capture_output=True,env={**os.environ,'NDLOCR_INTRA_THREADS':'1'})
 (folder/(image.stem+'.stderr')).write_bytes(p.stderr);assert p.returncode==0,(argv,p.stderr)
 return {'argv':argv,'exit_code':p.returncode,'elapsed_seconds':time.monotonic()-start}
with concurrent.futures.ThreadPoolExecutor(max_workers=4) as pool:r=list(pool.map(run,[(p,m) for p in images for m in ['deim','bands']]))
(root/'cli-runs.json').write_text(json.dumps(r,indent=2)+'\n');print('CLI controls completed',len(r))
