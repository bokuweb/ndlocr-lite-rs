"""Full saved Page evidence; no recognition or model loads."""
import importlib.util
from pathlib import Path
import tempfile
import unittest

spec=importlib.util.spec_from_file_location('page_packet_reader',Path(__file__).with_name('reproduce.py'))
reader=importlib.util.module_from_spec(spec);spec.loader.exec_module(reader)

class SavedPageEvidence(unittest.TestCase):
    def assert_invalid_artifact_rejected_before_dataset_access(self, *, same_binary=False, fresh=False):
        import hashlib,json
        with tempfile.TemporaryDirectory()as directory:
            root=Path(directory);scorer=root/'scorer.py';scorer.write_bytes(b'')
            (root/'page-stage.json').write_text(json.dumps(dict(fresh_candidate_artifacts_verified=True,scorer_sha256=hashlib.sha256(b'').hexdigest(),files_sha256={})))
            names=('ndlocr-lite-rs','capture_page_matrix')
            binding=dict(all_primary_artifacts_fresh_false=True,both_binaries_differ_from_baseline=True,candidate={name:dict(sha256='a'*64 if same_binary else'b'*64,fresh=fresh)for name in names},baseline={name:dict(sha256='a'*64)for name in names})
            (root/'page-artifact-binding.json').write_text(json.dumps(binding))
            # No dataset exists: rejection must be based on the actual artifact fields.
            with self.assertRaises(AssertionError):
                reader.verify_page(root,root/'output',scorer)

    def test_same_binary_is_rejected_even_if_manifest_declares_distinct(self):
        self.assert_invalid_artifact_rejected_before_dataset_access(same_binary=True)

    def test_fresh_cache_artifact_is_rejected_even_if_manifest_declares_rebuild(self):
        self.assert_invalid_artifact_rejected_before_dataset_access(fresh=True)

    def test_product_artifacts_are_fresh_and_differ_from_baseline(self):
        import json
        packet=Path(__file__).resolve().parent
        stage=json.loads((packet/'page-stage.json').read_text())
        self.assertTrue(stage['fresh_candidate_artifacts_verified'])
        binding=json.loads((packet/'page-artifact-binding.json').read_text())
        for name in ('ndlocr-lite-rs','capture_page_matrix'):
            self.assertFalse(binding['candidate'][name]['fresh'])
            self.assertNotEqual(binding['baseline'][name]['sha256'],binding['candidate'][name]['sha256'])

    def test_all_saved_page_routes_and_official_metrics_reproduce(self):
        packet=Path(__file__).resolve().parent
        # The frozen overlay is used only for the exact scorer source.
        with tempfile.TemporaryDirectory()as d:
            output=Path(d);manifest=reader.select_source(packet,output,None)
            result=reader.verify_page(packet,output,manifest.parent/'tools/eval_ocr_metrics.py')
            self.assertEqual(result['Page_input_IDs'],444)
            self.assertEqual(result['Page_unique_RGB'],443)
            self.assertGreater(result['Page_text_changes'],0)
            self.assertEqual(result['Page_per_input_regressions'],0)
            self.assertEqual(result['Page_independently_verified_CSV_rows'],21312)

if __name__=='__main__':unittest.main()
