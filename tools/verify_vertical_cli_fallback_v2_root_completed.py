"""Authenticate the actual Root bridge and its saved regressions without inference."""
import io
import json
import math
from pathlib import Path
import struct
from PIL import Image
from verify_additional_real_baseline import _packet
from verify_vertical_cli_fallback_v2 import verify_root_before
from verify_vertical_cli_full_completed import verify as verify_full, members
from vertical_cli_fallback_v2_full_matrix import FullBundleV2
from vertical_cli_fallback_v2_option_runner import require_root_v2_full_gate
from vertical_page_matrix import sha


def verify_quality_records(rows):
    assert len(rows)==3
    f32=lambda x:struct.unpack('!f',struct.pack('!f',x))[0]
    expected=[(False,f32(0.03)),(True,f32(0.03)),(True,-100.0)]
    for row,profile in zip(rows,expected,strict=True):
        assert type(row['quality_boost']) is bool and math.isfinite(row['quality_boost_min_delta'])
        assert (row['quality_boost'],f32(row['quality_boost_min_delta']))==profile
        assert row['initial_text'] and row['actual_text']==row['expected_text']
        assert row['actual_mean_bits']==row['expected_mean_bits']
        for name in ['initial_mean_bits','expected_mean_bits','actual_mean_bits']:
            bits=row[name];assert type(bits) is int and 0<=bits<2**32
            value=struct.unpack('!f',struct.pack('!I',bits))[0];assert math.isfinite(value) and 0<=value<=1
        x0,y0,x1,y1=row['bbox'];assert 0<=x0<x1 and 0<=y0<y1
        assert (row['crop_width'],row['crop_height'])==(x1-x0,y1-y0)
    forced=rows[-1]
    assert (forced['actual_text'],forced['actual_mean_bits'])!=(forced['initial_text'],forced['initial_mean_bits'])
    return len(rows)


def baseline_sources(folder):
    _,v2=_packet(folder,'before-nn-v1.index.json');gi,geometry=_packet(folder.parent/'real-vertical-primary-partition-probe','before-nn-v1.index.json');_,stop=_packet(folder.parent/'real-vertical-additional-stop-probe','before-nn-v1.index.json')
    ri,before=_packet(folder/'root-bridge-v1','baseline-before-green-v1.index.json')
    source={}
    for name,meta in json.loads(geometry['evidence/source-bridge-v1.json'])['source_files'].items():source[name]={'FVCv2':v2,'stop':stop,'this':geometry}[meta['packet']][meta['member']]
    root_source={}
    for name,meta in json.loads(before['evidence/source-bridge-v1.json'])['source_files'].items():
        raw={'geometry':source,'this':before}[meta['packet']][meta['member']];assert sha(raw)==meta['sha256'];root_source[name]=raw
    frozen=json.loads(before['evidence/root-baseline-frozen-v1.json']);assert {n:sha(b) for n,b in root_source.items()}==frozen['source_files']
    return ri,root_source,frozen,v2


def verify(folder):
    folder=Path(folder);verify_root_before(folder)
    full_folder=folder/'full-matrix-v1';replayed=verify_full(full_folder,FullBundleV2(full_folder));require_root_v2_full_gate({'complete':replayed['complete_workers']==56,**replayed})
    fi,full=_packet(full_folder,'completed-linux-v1.index.json');pi,proof=_packet(full_folder,'completed-reader-proof-v1.index.json')
    ri,before_source,baseline,v2=baseline_sources(folder);index,data=_packet(folder/'root-bridge-v1','completed-local-v2.index.json')
    assert index['before_Root_packet_sha256']==ri['sha256'] and index['full_completed_packet_sha256']==fi['sha256']
    assert index['native_executables_in_public_packet'] is False and index['new_model_calls_in_replay']==0
    comparison=members(next(raw for name,raw in full.items() if name.startswith('artifacts/') and '-comparison-' in name));applied=json.loads(data['evidence/root-runtime-applied-v1.json']);proposal=json.loads(data['evidence/root-runtime-proposed-v1.json'])
    assert applied['applied_to_Root'] and applied['full_gate_run']==39593424953 and not proposal['applied_to_Root'] and applied['files']==proposal['files']
    assert applied['full_summary_sha256']==sha(comparison['summary.json']) and applied['run_status_sha256']==sha(full['evidence/run.json'])
    assert applied['independent_replay_sha256']==sha(proof['evidence/full-independent-replay-v1.json']) and pi['completed_packet_sha256']==fi['sha256']
    assert applied['patch_sha256']==sha(data['evidence/root-runtime-proposed-v1.patch'])
    root=Path(baseline['binaries']['bin']['path']).parents[2];sources=dict(before_source)
    for name,meta in proposal['files'].items():
        raw=data['source/'+name];assert sha(raw)==meta['after_sha256']
        if name.endswith('.rs'):
            assert (sha(before_source[name]) if name in before_source else None)==meta['before_sha256'];sources[name]=raw
    meta3=json.loads(data['evidence/root-candidate-frozen-v3.json'])
    version_sources={3:sources|{name[7:]:raw for name,raw in data.items() if name.startswith('source/') and name[7:] in meta3['source_files']}}
    assert set(version_sources[3])==set(sources)|{'tests/support/vertical_cli_finish_known.rs','tests/fixtures/vertical_fallback_cli_finish/README.md','tests/fixtures/vertical_fallback_cli_finish/arial12.png','tests/fixtures/vertical_fallback_cli_finish/case.json','docs/benchmarks/vertical-cli-fallback-v2/options-plan-before-nn-v2.zip'}
    assert meta3['source_coverage_helper_sha256']==sha(data['source/tools/root_source_coverage.py'])
    option_index=json.loads((folder/'options-plan-before-nn-v2.index.json').read_text());assert sha(version_sources[3]['docs/benchmarks/vertical-cli-fallback-v2/options-plan-before-nn-v2.zip'])==option_index['sha256']
    bridge_raw=data['evidence/root-runtime-source-bridge-actual-v1.json'];bridge=json.loads(bridge_raw)
    assert bridge['actual_Root_runtime'] and bridge['src_Rust_files']==len(bridge['all_src_files'])==92 and bridge['audit_script_sha256']==sha(data['evidence/audit_root_runtime_source_v1.py'])
    changed=[]
    for name,meta in bridge['all_src_files'].items():
        old=v2['candidate-source/'+name];new=version_sources[3][name];expected=old
        assert sha(old)==meta['frozen_v2_sha256'] and sha(new)==meta['Root_sha256']
        if name=='src/infer/vertical_capacity.rs':expected=expected.replace(b'mod prefix;\n',b'mod prefix;\n\n#[cfg(test)]\n#[path = "../../tests/support/vertical_primary_partition_geometry.rs"]\nmod primary_partition_geometry;\n',1)
        if name=='src/infer/vertical_fallback.rs':
            expected=expected.replace(b'rgb.chunks_exact(3).enumerate()',b'rgb.as_chunks::<3>().0.iter().enumerate()',1)
            for block in [b'\n#[cfg(test)]\n#[path = "../../tests/support/vertical_fallback_capture.rs"]\nmod capture;\n',b'\n#[cfg(test)]\n#[path = "../../tests/support/vertical_cli_geometry.rs"]\nmod cli_geometry_capture;\n']:expected=expected.replace(block,b'',1)
        assert expected==new,name
        if old!=new:changed.append(name)
    assert changed==bridge['different_files']==['src/infer/vertical_capacity.rs','src/infer/vertical_fallback.rs']
    assets=json.loads((folder.parent/'vertical-page-fallback/bundle.json').read_text())['assets'];asset_hashes={a['name']:a['sha256'] for a in assets.values()}
    binaries={};metas={}
    def builds(meta,label,source,feature):
        assert meta['source_files']=={n:sha(b) for n,b in source.items()} and len(source)==852
        logs={name:raw for name,raw in data.items() if name.startswith('evidence/root-'+label) and name.endswith('.jsonl')};events=[]
        for raw in logs.values():events.extend(json.loads(line) for line in raw.decode().splitlines() if line.startswith('{'))
        assert meta['own_artifacts'] and all(e in events and e['fresh'] is False and ('onnx' in e['features'])==feature for e in meta['own_artifacts'])
        for value in meta['binaries'].values():
            assert value['compiler_artifact'] in meta['own_artifacts'] and any(sha(raw)==value['build_stdout_sha256'] for raw in logs.values())
            binaries[value['path']]=value['sha256']
    for version in [3]:
        meta=json.loads(data[f'evidence/root-candidate-frozen-v{version}.json']);metas[version]=meta
        assert meta['root_patch_receipt_sha256']==sha(data['evidence/root-runtime-applied-v1.json']) and meta['runtime_source_bridge_sha256']==sha(bridge_raw)
        builds(meta,'candidate-',version_sources[version],True)
    old_index,old_data=_packet(folder/'root-bridge-v1','completed-local-v1.index.json');assert index['old_incomplete_packet_sha256']==old_index['sha256']
    old_meta=json.loads(old_data['evidence/root-candidate-frozen-v2.json'])
    assert all(metas[3]['binaries'][key]['sha256']==old_meta['binaries'][key]['sha256'] for key in ['bin','test','libtest'])
    assert metas[3]['binaries']['test']['sha256']==baseline['binaries']['test']['sha256']
    default=json.loads(data['evidence/root-default-candidate-frozen-v2.json']);builds(default,'default-candidate-',version_sources[3],False)
    old_default=json.loads(data['evidence/root-default-baseline-frozen-v1.json']);_,old_packet=_packet(folder/'root-bridge-v1','default-baseline-v1.index.json');assert data['evidence/root-default-baseline-frozen-v1.json']==old_packet['evidence/root-default-baseline-frozen-v1.json'];binaries[old_default['path']]=old_default['sha256']
    def bindings(values,source):
        for absolute,digest in values.items():
            name=str(Path(absolute).relative_to(root))
            if absolute in binaries:assert digest==binaries[absolute];continue
            if name in source:raw=source[name]
            elif name.startswith('models/'):assert digest==asset_hashes[Path(name).name];continue
            elif name.startswith('tmp/vertical-cli-fallback-v2/root-feature-disabled-v2/'):raw=data['disabled/'+name.split('/root-feature-disabled-v2/',1)[1]]
            else:raw=data['evidence/'+Path(name).name]
            assert sha(raw)==digest,name
    case=json.loads(before_source['tests/fixtures/vertical_fallback_page/case.json']);literal=case['expected'].encode()
    for version in [3]:
        prefix=f'native-v{version}/';pre=json.loads(data[prefix+'before-NN.json']);run=json.loads(data[prefix+'command.json']);meta=metas[version]
        bindings(run['files_sha256'],version_sources[version]);assert pre['files_sha256']==run['files_sha256'] and not pre['complete'] and run['complete'] is True
        assert run['commands_planned']==pre['commands_planned'] and len(run['commands'])==3
        for command,planned in zip(run['commands'],pre['commands_planned']):assert all(command[k]==v for k,v in planned.items()) and command['frozen_files_after_unchanged']
        assert run['commands'][0]['exit_code']==0 and b'2 passed; 0 failed;' in data[prefix+'CLI.stdout']
        assert run['commands'][0]['argv']==[meta['binaries']['test']['path'],'--ignored','--nocapture','--test-threads=1']
        assert run['commands'][0]['env']['NDLOCR_TEST_CLI_BINARY']==meta['binaries']['bin']['path']
        for deim in [False,True]:
            cp=prefix+'CLI-evidence/deim-'+str(deim).lower()+'/';command=json.loads(data[cp+'command.json']);argv=command['argv']
            assert command['binary']==meta['binaries']['bin']['path'] and command['exit_code']==0 and command['deim']==deim
            assert data[cp+'body.txt']==literal and data[cp+'stdout']==literal+b'\n' and not data[cp+'stderr']
            assert argv[argv.index('--use-deim-detection')+1]==str(deim).lower() and argv[argv.index('--enable-cascade')+1]=='false' and argv[argv.index('--line-crop-padding')+1]=='0'
        assert all(c['exit_code']==0 for c in run['commands']) and b'1 passed; 0 failed;' in data[prefix+'known-quality.stdout'] and b'468 passed; 0 failed; 114 ignored;' in data[prefix+'ONNX-nonignored-lib.stdout']
    rows=json.loads(data['native-v3/known-quality-evidence/results.json']);settings=verify_quality_records(rows)
    image=Image.open(io.BytesIO(version_sources[3]['tests/fixtures/vertical_fallback_cli_finish/arial12.png'])).convert('RGB')
    for i,row in enumerate(rows,1):assert data[f'native-v3/known-quality-evidence/crop-{i}.rgb']==image.crop(row['bbox']).tobytes()
    pre=json.loads(data['disabled/before-CLI.json']);bindings(pre['files_sha256'],version_sources[3]);assert pre['CLI_processes_planned']==16 and pre['NN_calls']==pre['model_loads']==0 and not pre['complete']
    commands=json.loads(data['disabled/commands.json']);assert len(commands)==16;values={}
    for command in commands:
        variant,asset,name,deim=[command[k] for k in ['variant','asset','case','deim']];label=f'{variant}-{asset}-{name}-{deim}';stdout=data['disabled/'+label+'.stdout'];stderr=data['disabled/'+label+'.stderr'];body=data.get('disabled/'+label+'.txt');argv=command['argv']
        binary=old_default['path'] if variant=='baseline' else default['binaries']['bin']['path'];assert argv[0]==binary and command['binary_sha256']==binaries[binary]
        assert sha(stdout)==command['stdout_sha256'] and sha(stderr)==command['stderr_sha256'] and command['body_exists']==(body is not None) and command['body_sha256']==(sha(body) if body is not None else None)
        assert argv[argv.index('--use-deim-detection')+1]==str(deim).lower();expected_image=root/'tests/fixtures/vertical_fallback_page/arial121.png' if name=='body' else root/'tmp/vertical-cli-fallback-v2/root-feature-disabled-v2/white.png';assert argv[argv.index('--image')+1]==str(expected_image)
        image_bytes=before_source['tests/fixtures/vertical_fallback_page/arial121.png'] if name=='body' else data['disabled/white.png'];assert sha(image_bytes)==command['image_sha256']
        value=(command['exit_code'],stdout,stderr,body);assert (variant,asset,name,deim) not in values;values[variant,asset,name,deim]=value
        if name=='body':assert value[0]==1 and not stdout and body is None and (b'Error: file not found' if asset=='missing' else b'Error: onnx feature is disabled') in stderr
        else:assert value[0]==0 and stdout==b'\n' and body==b''
    for asset in ['missing','dummy']:
        for name in ['body','white']:
            for deim in [False,True]:assert values['baseline',asset,name,deim]==values['candidate',asset,name,deim]
    summary=json.loads(data['disabled/summary.json']);assert summary['CLI_processes']==16 and summary['cases']==8 and summary['identical_exit_stdout_stderr_body'] and summary['frozen_bytes_after_unchanged'] and summary['NN_calls']==summary['model_loads']==0
    assert b'456 passed; 0 failed; 39 ignored;' in data['disabled/default-lib.stdout']
    return {'Root_CLI_literal_Green':2,'quality_settings':settings,'disabled_CLI_processes':16,'source_files':len(version_sources[3]),'ONNX_nonignored_passed':468,'default_nonignored_passed':456,'new_model_calls_in_replay':0}


if __name__=='__main__':
    print(json.dumps(verify(Path(__file__).resolve().parents[1]/'docs/benchmarks/vertical-cli-fallback-v2'),indent=2))
