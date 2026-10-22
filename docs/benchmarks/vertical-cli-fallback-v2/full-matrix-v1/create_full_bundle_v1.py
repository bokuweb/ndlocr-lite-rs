from pathlib import Path
import hashlib,json,sys
sys.path.insert(0,'tools')
from verify_additional_real_baseline import _packet
from vertical_cli_fallback_v2_full_scope import combine_scope
from vertical_cli_fallback_full_matrix import full_plan
repo=Path.cwd();folder=repo/'docs/benchmarks/vertical-cli-fallback-v2/full-matrix-v1';folder.mkdir();parent=folder.parent;old=repo/'docs/benchmarks/vertical-cli-fallback/full-matrix-v1';sha=lambda q:hashlib.sha256(Path(q).read_bytes()).hexdigest();previous=json.loads((old/'bundle.json').read_text());_,packet=_packet(parent,'before-nn-v1.index.json');before=json.loads(packet['evidence/before-NN-v1.json']);manifest=json.loads(packet['evidence/new-manifest.json']);images,scores=combine_scope(previous,manifest)
protocol=repo/'docs/vertical_cli_fallback_v2_full_gate.md';(folder/'protocol-before-nn.md').write_bytes(protocol.read_bytes());plan=full_plan(images,scores)
index={'schema':2,'previous_bundle_sha256':sha(old/'bundle.json'),'before_packet_sha256':json.loads((parent/'before-nn-v1.index.json').read_text())['sha256'],'new72_packet_sha256':json.loads((parent/'new72-cli-matrix-v1.index.json').read_text())['sha256'],'protocol_sha256':sha(protocol),'source_files':{'baseline':before['baseline_source_files'],'candidate':before['source_files']},'assets':previous['assets'],'physical_inputs':images,'scoring_inputs':scores,'shards':8,'expected':{'physical_inputs':1311,'scoring_inputs':1317,'unique_RGB':1310,'image_results':26220,'CSV_rows':79020,'PageRecognizer_loads':48,'explicit_Page_model_session_loads':96}}
(folder/'bundle.json').write_text(json.dumps(index,indent=2)+'\n');(folder/'plan.json').write_text(json.dumps(plan,indent=2)+'\n');(folder/'create_full_bundle_v1.py').write_bytes(Path(__file__).read_bytes());print({'physical':len(images),'scoring':len(scores),'jobs':len(plan['jobs']),'NN_calls':0})
