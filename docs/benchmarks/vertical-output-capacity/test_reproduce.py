import tempfile
import unittest
from pathlib import Path
import reproduce

class ReplayBoundaryTests(unittest.TestCase):
    def test_existing_file_cannot_be_overwritten(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / 'output'
            path.mkdir()
            sentinel = path / 'keep.txt'
            sentinel.write_bytes(b'keep\n\n')
            with self.assertRaises(ValueError):
                reproduce.empty_output(path)
            self.assertEqual(sentinel.read_bytes(), b'keep\n\n')

    def test_archive_paths_cannot_escape_or_alias(self):
        for path in ['/absolute', '../parent', 'ok/../../parent', 'a/../b', './alias', 'a//b', 'back\\slash']:
            with self.subTest(path=path), self.assertRaises(ValueError):
                reproduce.relative_member(path)
        self.assertEqual(reproduce.relative_member('tmp/a/b.rgb'), Path('tmp/a/b.rgb'))

if __name__ == '__main__':
    unittest.main()
