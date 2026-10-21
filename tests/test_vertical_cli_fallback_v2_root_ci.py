from pathlib import Path
import io
import json
import sys
import unittest
from unittest.mock import patch
import zipfile
sys.path.insert(0,str(Path(__file__).resolve().parents[1]/'tools'))


class RootCiPacketTest(unittest.TestCase):
    def test_both_platforms_and_all_actual_cli_outputs_are_verified_without_models(self):
        from verify_vertical_cli_fallback_v2_root_ci import verify
        result=verify(Path(__file__).resolve().parents[1]/'docs/benchmarks/vertical-cli-fallback-v2/root-bridge-v1')
        self.assertEqual(result,{'successful_jobs':5,'actual_CLI_results':4,'known_quality_platforms':2,'NN_calls_in_replay':0})

    def test_incomplete_jobs_wrong_body_or_skipped_quality_are_rejected(self):
        import verify_vertical_cli_fallback_v2_root_ci as reader
        from verify_additional_real_baseline import _packet
        from vertical_page_matrix import sha
        folder=Path(__file__).resolve().parents[1]/'docs/benchmarks/vertical-cli-fallback-v2/root-bridge-v1';index,data=_packet(folder,'completed-ci-v1.index.json')
        for failure in ['job_missing','body_changed','quality_missing']:
            changed=dict(data)
            if failure=='job_missing':
                jobs=json.loads(changed['evidence/jobs.json']);jobs['jobs'].pop();changed['evidence/jobs.json']=json.dumps(jobs).encode()
            elif failure=='quality_missing':
                changed['logs/linux-attempt2.log']=b'\n'.join(line for line in data['logs/linux-attempt2.log'].splitlines() if b'app::vertical_cli_finish_known::accepted_vertical_finishes_like_existing_page_batch_with_quality_options' not in line)
            else:
                name=next(n for n in data if n.startswith('artifacts/') and 'ubuntu' in n);buffer=io.BytesIO()
                with zipfile.ZipFile(io.BytesIO(data[name])) as old,zipfile.ZipFile(buffer,'w',compression=zipfile.ZIP_DEFLATED) as new:
                    for member in old.namelist():
                        raw=old.read(member)
                        if member.endswith('body.txt'):raw=b'changed'
                        if member.endswith('stdout'):raw=b'changed\n'
                        new.writestr(member,raw)
                changed[name]=buffer.getvalue();metadata=json.loads(changed['evidence/artifacts.json']);artifact=next(a for a in metadata['artifacts'] if name=='artifacts/'+a['name']+'.zip');artifact.update(size_in_bytes=len(changed[name]),digest='sha256:'+sha(changed[name]));changed['evidence/artifacts.json']=json.dumps(metadata).encode()
            with self.subTest(failure=failure),patch.object(reader,'_packet',return_value=(index,changed)),self.assertRaises(AssertionError):reader.verify(folder)
