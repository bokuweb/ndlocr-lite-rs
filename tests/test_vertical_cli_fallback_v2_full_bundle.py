from pathlib import Path
import sys
import tempfile
import unittest
sys.path.insert(0,str(Path(__file__).resolve().parents[1]/'tools'))


class FullBundleV2Test(unittest.TestCase):
    def bundle(self):
        from vertical_cli_fallback_v2_full_matrix import FullBundleV2
        return FullBundleV2(Path(__file__).resolve().parents[1]/'docs/benchmarks/vertical-cli-fallback-v2/full-matrix-v1')

    def test_actual_source_and_all_old_aliases_and_new_controls_authenticate(self):
        bundle=self.bundle()
        self.assertTrue(bundle.baseline_FVB_bridge)
        self.assertTrue(bundle.new72_gate['matrix_gate_pass'])
        self.assertEqual(len(bundle.sources['baseline']),282)
        self.assertEqual(len(bundle.sources['candidate']),848)
        self.assertEqual((len(bundle.index['physical_inputs']),len(bundle.index['scoring_inputs'])),(1311,1317))
        self.assertEqual(bundle.plan['expected_CSV_rows'],79020)

    def test_reused_materializer_preserves_png_jpeg_bytes_and_passes_only_images(self):
        from vertical_page_matrix import sha
        bundle=self.bundle();rows=bundle.index['physical_inputs']
        selected=[next(r for r in rows if r['image'].lower().endswith(('.jpg','.jpeg'))),rows[-1]]
        with tempfile.TemporaryDirectory() as directory:
            manifest=bundle.materialize_inputs(Path(directory),selected)
            for actual,expected in zip(manifest['inputs'],selected,strict=True):
                self.assertEqual(set(actual),{'id','image'})
                self.assertEqual(sha(Path(actual['image']).read_bytes()),expected['image_sha256'])
                self.assertEqual(Path(actual['image']).suffix.lower(),Path(expected['image']).suffix.lower())

    def test_frozen_harness_covers_the_new_loader_reader_and_pillow_dependency_pin(self):
        import vertical_cli_fallback_full_matrix as original
        from vertical_cli_fallback_v2_full_matrix import configure_harness
        before=original.HARNESS_FILES
        try:
            configure_harness()
            self.assertTrue(set(before)<=set(original.HARNESS_FILES))
            for name in ['tools/vertical_cli_fallback_v2_full_matrix.py','tools/verify_vertical_cli_fallback_v2.py','tools/vertical_cli_fallback_v2_full_scope.py','tools/low_stop_evidence_requirements.txt']:
                self.assertIn(name,original.HARNESS_FILES)
            self.assertEqual(len(set(original.HARNESS_FILES)),len(original.HARNESS_FILES))
        finally:original.HARNESS_FILES=before
