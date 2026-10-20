"""Full saved Page evidence; no recognition or model loads."""
import importlib.util
from pathlib import Path
import tempfile
import unittest

spec=importlib.util.spec_from_file_location('page_packet_reader',Path(__file__).with_name('reproduce.py'))
reader=importlib.util.module_from_spec(spec);spec.loader.exec_module(reader)

class SavedPageEvidence(unittest.TestCase):
    def test_all_saved_page_routes_and_official_metrics_reproduce(self):
        packet=Path(__file__).resolve().parent
        # The frozen overlay is used only for the exact scorer source.
        with tempfile.TemporaryDirectory()as d:
            output=Path(d);manifest=reader.select_source(packet,output,None)
            result=reader.verify_page(packet,output,manifest.parent/'tools/eval_ocr_metrics.py')
            self.assertEqual(result['Page_input_IDs'],444)
            self.assertEqual(result['Page_unique_RGB'],443)
            self.assertEqual(result['Page_text_changes'],0)
            self.assertEqual(result['Page_independently_verified_CSV_rows'],21312)

if __name__=='__main__':unittest.main()
