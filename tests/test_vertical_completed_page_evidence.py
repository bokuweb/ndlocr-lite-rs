import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest


class CompletedPageEvidenceTest(unittest.TestCase):
    def test_saved_page_packet_replays_without_models_and_never_claims_full_gate(self):
        repo=Path(__file__).resolve().parents[1]
        with tempfile.TemporaryDirectory() as directory:
            output=Path(directory)/'replayed'
            result=subprocess.run([sys.executable,str(repo/'docs/benchmarks/vertical-page-fallback/verify_completed_pages.py'),'--output',str(output)],cwd=directory,capture_output=True,text=True)
            self.assertEqual(result.returncode,0,result.stderr)
            report=json.loads(result.stdout)
            self.assertEqual(report['valid_Page_jobs'],6)
            self.assertEqual(report['saved_Page_results'],11052)
            self.assertEqual(report['independently_verified_CSV_rows'],33192)
            self.assertEqual(report['per_model_raw_gain_pairs'],{'30':35,'50':35,'100':8})
            self.assertEqual(report['per_input_regressions'],0)
            self.assertEqual(report['replayed_model_choice_observations'],424)
            self.assertEqual(report['new_NN_calls'],0)
            self.assertFalse(report['full_gate_pass'])
            self.assertEqual(report,json.loads((output/'verification.json').read_text()))


if __name__=='__main__':
    unittest.main()
