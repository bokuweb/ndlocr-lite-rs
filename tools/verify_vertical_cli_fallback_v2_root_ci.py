"""Verify completed platform CI and original CLI artifacts without recognition."""
import io
import json
from pathlib import Path
import re
import zipfile
from verify_additional_real_baseline import _packet
from vertical_page_matrix import sha


def verify(folder):
    folder=Path(folder);index,data=_packet(folder,'completed-ci-v1.index.json');run=json.loads(data['evidence/run.json']);jobs=json.loads(data['evidence/jobs.json'])
    expected={'rust (ubuntu-latest)','rust (macos-latest)','rust (windows-latest)','real PARSeq (ubuntu-latest)','real PARSeq (macos-latest)'}
    assert index['head']==run['headSha']=='0a0f421c0dd6f564f0ab09ccf2ca46836e29b349' and index['run']==run['databaseId']==39627691771
    assert run['status']=='completed' and run['conclusion']=='success' and run['attempt']==2
    assert len(jobs['jobs'])==jobs['total_count']==5 and {j['name'] for j in jobs['jobs']}==expected
    assert all(j['status']=='completed' and j['conclusion']=='success' for j in jobs['jobs'])
    root_index=json.loads((folder/'completed-local-v2.index.json').read_text());assert index['root_completed_packet_sha256']==root_index['sha256']
    frozen_raw=data['evidence/root-frozen.json'];assert sha(frozen_raw)==root_index['members']['evidence/root-candidate-frozen-v3.json']['sha256']
    frozen=json.loads(frozen_raw);case_raw=data['fixture-case.json'];assert sha(case_raw)==frozen['source_files']['tests/fixtures/vertical_fallback_page/case.json']
    case=json.loads(case_raw);body=case['expected'].encode();assert len(case['expected'])==120 and sha(body)==case['expected_sha256']
    artifacts=json.loads(data['evidence/artifacts.json']);assert artifacts['total_count']==len(artifacts['artifacts'])==2
    count=0
    for os,system in [('linux','ubuntu'),('mac','macos')]:
        name=f'root-fvc-cli-{system}-latest-attempt2';artifact,=[a for a in artifacts['artifacts'] if a['name']==name];raw=data['artifacts/'+name+'.zip']
        assert artifact['workflow_run']['id']==index['run'] and artifact['workflow_run']['head_sha']==index['head']
        assert len(raw)==artifact['size_in_bytes'] and 'sha256:'+sha(raw)==artifact['digest']
        with zipfile.ZipFile(io.BytesIO(raw)) as archive:
            assert set(archive.namelist())=={f'deim-{deim}/{name}' for deim in ['false','true'] for name in ['body.txt','stdout','stderr','command.json']}
            for deim in [False,True]:
                prefix='deim-'+str(deim).lower()+'/';command=json.loads(archive.read(prefix+'command.json'));argv=command['argv']
                assert command['exit_code']==0 and command['deim']==deim
                assert archive.read(prefix+'body.txt')==body and archive.read(prefix+'stdout')==body+b'\n' and not archive.read(prefix+'stderr')
                assert argv[:2]==['recognize-page','--image'] and Path(argv[2]).name==case['image']
                for flag,value in [('--use-deim-detection',str(deim).lower()),('--enable-cascade','false'),('--structure-rules','false'),('--line-crop-padding','0')]:assert argv[argv.index(flag)+1]==value
                assert Path(argv[argv.index('--model')+1]).name=='parseq-ndl-24x256-30-tiny-189epoch-tegaki3-r8data-202604.onnx'
                assert Path(argv[argv.index('--charset')+1]).name=='NDLmoji.yaml';count+=1
        log=re.sub(r'\x1b\[[0-?]*[ -/]*[@-~]','',data[f'logs/{os}-attempt2.log'].decode())
        assert index['head'] in log
        for test in ['app::vertical_cli_finish_known::accepted_vertical_finishes_like_existing_page_batch_with_quality_options','supported_vertical_cli_fallback_restores_literal_body_deim_off','supported_vertical_cli_fallback_restores_literal_body_deim_on']:assert 'test '+test+' ... ok' in log
        assert b'TimeoutError: The read operation timed out' in data[f'logs/{os}-attempt1.log']
    assert index['NN_calls_in_replay']==0
    return {'successful_jobs':5,'actual_CLI_results':count,'known_quality_platforms':2,'NN_calls_in_replay':0}


if __name__=='__main__':
    print(json.dumps(verify(Path(__file__).resolve().parents[1]/'docs/benchmarks/vertical-cli-fallback-v2/root-bridge-v1'),indent=2))
