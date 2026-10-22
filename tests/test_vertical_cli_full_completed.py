from pathlib import Path
import sys
import unittest
from unittest.mock import patch
import io
import json
import zipfile
sys.path.insert(0,str(Path(__file__).resolve().parents[1]/'tools'))


class CompletedFVCFullTest(unittest.TestCase):
    def test_missing_partial_or_different_build_workers_cannot_form_a_complete_gate(self):
        from verify_additional_real_baseline import _packet
        from vertical_page_matrix import sha
        import verify_vertical_cli_full_completed as reader
        folder=Path(__file__).resolve().parents[1]/'docs/benchmarks/vertical-cli-fallback/full-matrix-v1'
        index,data=_packet(folder,'completed-linux-v1.index.json')
        name=next(n for n in data if n.startswith('artifacts/') and '-result-' in n)
        for failure in ['missing','partial','different_build']:
            with self.subTest(failure=failure):
                changed=dict(data);download=json.loads(data['evidence/downloaded.json'])
                artifact=next(a for a in download['artifacts'] if name=='artifacts/'+a['name']+'.zip')
                if failure=='missing':
                    del changed[name];download['artifacts'].remove(artifact)
                else:
                    entries=reader.members(data[name]);meta=json.loads(entries['evidence/job.json'])
                    if failure=='partial':meta['complete']=False
                    else:meta['build_receipt_sha256']='0'*64
                    entries['evidence/job.json']=json.dumps(meta).encode();buffer=io.BytesIO()
                    with zipfile.ZipFile(buffer,'w',compression=zipfile.ZIP_DEFLATED) as z:
                        for key,raw in entries.items():z.writestr(key,raw)
                    changed[name]=buffer.getvalue();artifact.update(bytes=len(changed[name]),sha256=sha(changed[name]))
                changed['evidence/downloaded.json']=json.dumps(download).encode()
                # Reauthenticate the mutated transport to exercise semantic coverage checks.
                with patch.object(reader,'_packet',return_value=(index,changed)),self.assertRaises(ValueError):reader.verify(folder)

    def test_saved_v1_replays_all_workers_scores_and_page_confidence_bits(self):
        from verify_vertical_cli_full_completed import verify
        result=verify(Path(__file__).resolve().parents[1]/'docs/benchmarks/vertical-cli-fallback/full-matrix-v1')
        self.assertEqual(result['complete_workers'],56)
        self.assertEqual(result['image_results'],24780)
        self.assertEqual(result['verified_CSV_rows'],74700)
        self.assertEqual(result['scoring_comparison_pairs'],37350)
        self.assertEqual(result['regression_pairs'],0)
        self.assertEqual(result['page_body_bbox_confidence_changes'],0)
        self.assertEqual(result['raw_known_gain_pairs'],798)
        self.assertTrue(result['full_gate_pass'])
        self.assertEqual(result['new_model_calls_in_replay'],0)
        self.assertFalse(result['native_bytes_in_public_packet'])
