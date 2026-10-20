"""Model-free checks of isolated source restoration and archive integrity."""
import importlib.util
import hashlib
import json
from pathlib import Path
import tempfile
import unittest
import zipfile

spec=importlib.util.spec_from_file_location('vertical_partition_reader',Path(__file__).with_name('reproduce.py'))
reader=importlib.util.module_from_spec(spec);spec.loader.exec_module(reader)

class FrozenSource(unittest.TestCase):
    def packet(self, directory, entries=None, corrupt=False):
        entries=entries or {'Cargo.toml':b'[package]\nname="fixture"\nversion="0.0.0"\n','tools/eval_ocr_metrics.py':b'# fixed scorer\n'}
        packet=directory/'packet';packet.mkdir()
        with zipfile.ZipFile(packet/'source-overlay.zip','w')as z:
            for name,data in entries.items():z.writestr(name,data)
        index={name:dict(sha256=hashlib.sha256(data).hexdigest(),bytes=len(data))for name,data in entries.items()}
        if corrupt:index['Cargo.toml']['sha256']='0'*64
        (packet/'source-overlay.json').write_text(json.dumps(dict(members=index)))
        return packet

    def test_default_source_and_scorer_are_restored_under_output_without_cwd_dependency(self):
        with tempfile.TemporaryDirectory()as directory:
            root=Path(directory);packet=self.packet(root);output=root/'run';output.mkdir()
            manifest=reader.select_source(packet,output,None)
            self.assertEqual(manifest,output/'replay-source/Cargo.toml')
            self.assertEqual((manifest.parent/'tools/eval_ocr_metrics.py').read_bytes(),b'# fixed scorer\n')
            self.assertFalse((root/'Cargo.toml').exists())

    def test_explicit_manifest_does_not_restore_or_replace_source(self):
        with tempfile.TemporaryDirectory()as directory:
            root=Path(directory);packet=self.packet(root);output=root/'run';output.mkdir();existing=root/'project';existing.mkdir();manifest=existing/'Cargo.toml';manifest.write_bytes(b'caller project')
            self.assertEqual(reader.select_source(packet,output,manifest),manifest.resolve())
            self.assertEqual(manifest.read_bytes(),b'caller project')
            self.assertFalse((output/'replay-source').exists())

    def test_unsafe_members_are_rejected_before_any_extraction(self):
        for name in ('../escape','/absolute','safe/../../escape'):
            with self.subTest(name=name),tempfile.TemporaryDirectory()as directory:
                root=Path(directory);packet=self.packet(root,{'Cargo.toml':b'ok',name:b'bad'});output=root/'run';output.mkdir()
                with self.assertRaises((AssertionError,ValueError)):reader.select_source(packet,output,None)
                self.assertFalse((output/'replay-source').exists())

    def test_hash_mismatch_is_rejected_before_any_extraction(self):
        with tempfile.TemporaryDirectory()as directory:
            root=Path(directory);packet=self.packet(root,corrupt=True);output=root/'run';output.mkdir()
            with self.assertRaises((AssertionError,ValueError)):reader.select_source(packet,output,None)
            self.assertFalse((output/'replay-source').exists())


class PrebuiltReplay(unittest.TestCase):
    def test_primary_recorded_binary_is_allowed(self):
        with tempfile.TemporaryDirectory()as d:
            root=Path(d);binary=root/'recorded';binary.write_bytes(b'recorded')
            (root/'prebuilt-replay-binary.json').write_text(json.dumps(dict(sha256=hashlib.sha256(b'recorded').hexdigest())))
            self.assertEqual(reader.select_prebuilt_binary(root,binary),binary.resolve())
    def test_additional_verified_pure_binary_is_allowed(self):
        with tempfile.TemporaryDirectory()as d:
            root=Path(d);binary=root/'pure';binary.write_bytes(b'pure')
            (root/'prebuilt-replay-binary.json').write_text(json.dumps(dict(sha256='0'*64,also_allowed=[dict(sha256=hashlib.sha256(b'pure').hexdigest())])))
            self.assertEqual(reader.select_prebuilt_binary(root,binary),binary.resolve())
    def test_unknown_binary_is_rejected_without_running_it(self):
        with tempfile.TemporaryDirectory()as d:
            root=Path(d);binary=root/'unknown';binary.write_bytes(b'unknown')
            (root/'prebuilt-replay-binary.json').write_text(json.dumps(dict(sha256='0'*64)))
            with self.assertRaises(AssertionError):reader.select_prebuilt_binary(root,binary)
            self.assertEqual(binary.read_bytes(),b'unknown')

if __name__=='__main__':unittest.main()
