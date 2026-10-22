from pathlib import Path
import sys
import tempfile
import unittest
sys.path.insert(0,str(Path(__file__).resolve().parents[1]/'tools'))


class RootSourceCoverageTest(unittest.TestCase):
    def test_late_support_fixtures_and_literal_include_resources_are_never_omitted(self):
        from root_source_coverage import source_files
        with tempfile.TemporaryDirectory() as directory:
            root=Path(directory)
            files={'Cargo.toml':'','Cargo.lock':'','src/lib.rs':'','tests/support/late.rs':'','tests/fixtures/late.png':'pixels','examples/a.rs':'','benches/a.rs':'','tests/cli.rs':'let _ = include_bytes!("../docs/plan.zip");','docs/plan.zip':'frozen','tools/unrelated.py':'ignored'}
            for name,text in files.items():path=root/name;path.parent.mkdir(parents=True,exist_ok=True);path.write_text(text)
            expected=set(files)-{'tools/unrelated.py'}
            self.assertEqual(set(source_files(root)),expected)

    def test_missing_external_or_dynamic_includes_cannot_silently_shrink_the_freeze(self):
        from root_source_coverage import source_files
        with tempfile.TemporaryDirectory() as directory:
            root=Path(directory);(root/'src').mkdir();(root/'Cargo.toml').touch();(root/'Cargo.lock').touch()
            for code in ['include_bytes!("missing.png");','include_str!("../../outside.txt");','include_bytes!(concat!("a", "b"));']:
                (root/'src/lib.rs').write_text(code)
                with self.subTest(code=code),self.assertRaises((ValueError,FileNotFoundError)):source_files(root)
