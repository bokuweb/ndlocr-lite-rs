import gzip
import hashlib
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest
import zipfile

spec=importlib.util.spec_from_file_location('partition_history',Path(__file__).with_name('history_reader.py'))
reader=importlib.util.module_from_spec(spec);spec.loader.exec_module(reader)

class HistoryTests(unittest.TestCase):
    def packet(self,root,corrupt=False,unsafe=False):
        blobs={'snapshot source':b'fn main() {}\n','original pixels':b'\x00\x7f\xff'};backers={};logical={}
        for archive,member,label,path in [('history.zip','objects/source','snapshot source','tmp/phase/source/src/lib.rs'),('inputs.zip','input.rgb','original pixels','tmp/phase/input.rgb')]:
            data=blobs[label];h=hashlib.sha256(data).hexdigest()
            with zipfile.ZipFile(root/archive,'w')as z:z.writestr(member,data if not corrupt else data+b'x')
            backers[h]=dict(archive=archive,member=member,bytes=len(data));logical[path]=dict(sha256=h,bytes=len(data))
        if unsafe:logical['../escape']=next(iter(logical.values()))
        index=dict(logical_files=logical,backing_objects=backers)
        (root/'history-index.json.gz').write_bytes(gzip.compress(json.dumps(index).encode()))
        return index
    def test_verifies_content_addressed_objects_and_dataset_aliases(self):
        with tempfile.TemporaryDirectory()as d:
            root=Path(d);self.packet(root);result=reader.verify_history(root)
            self.assertEqual(result['logical_files'],2);self.assertEqual(result['backing_objects'],2)
    def test_changed_object_bytes_are_rejected(self):
        with tempfile.TemporaryDirectory()as d:
            root=Path(d);self.packet(root,corrupt=True)
            with self.assertRaises((AssertionError,ValueError)):reader.verify_history(root)
    def test_selected_snapshot_restores_original_paths_without_other_inputs(self):
        with tempfile.TemporaryDirectory()as d:
            root=Path(d);self.packet(root);output=root/'restored';reader.restore_history(root,output,'tmp/phase/source')
            self.assertEqual((output/'tmp/phase/source/src/lib.rs').read_bytes(),b'fn main() {}\n')
            self.assertFalse((output/'tmp/phase/input.rgb').exists())
    def test_unsafe_logical_path_is_rejected_before_output_is_created(self):
        with tempfile.TemporaryDirectory()as d:
            root=Path(d);self.packet(root,unsafe=True);output=root/'restored'
            with self.assertRaises((AssertionError,ValueError)):reader.restore_history(root,output,'')
            self.assertFalse(output.exists())

if __name__=='__main__':unittest.main()
