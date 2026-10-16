"""固定資産の取得と破損拒否をネットワークなしで検証する。"""
import hashlib
import importlib.util
import tempfile
import unittest
from pathlib import Path


class FetchAssetsTests(unittest.TestCase):
    def setUp(self):
        spec = importlib.util.spec_from_file_location("fetch", Path(__file__).resolve().parents[1] / "tools/fetch_upstream_assets.py")
        self.fetch = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(self.fetch)

    def test_verified_asset_and_offline_revalidation(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source = root / "source"
            source.write_bytes(b"model")
            asset = {"name": "model.onnx", "url": source.as_uri(), "sha256": hashlib.sha256(b"model").hexdigest()}
            target = root / "out"
            self.fetch.fetch_asset(asset, target)
            source.unlink()
            self.fetch.fetch_asset(asset, target, offline=True)
            (target / "model.onnx").write_bytes(b"broken")
            with self.assertRaises(ValueError):
                self.fetch.fetch_asset(asset, target, offline=True)

    def test_bad_download_preserves_existing_file(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source = root / "source"
            source.write_bytes(b"bad")
            output = root / "out"
            output.mkdir()
            (output / "model.onnx").write_bytes(b"existing")
            asset = {"name": "model.onnx", "url": source.as_uri(), "sha256": "0" * 64}
            with self.assertRaises(ValueError):
                self.fetch.fetch_asset(asset, output)
            self.assertEqual((output / "model.onnx").read_bytes(), b"existing")
            self.assertEqual(list(output.iterdir()), [output / "model.onnx"])


if __name__ == "__main__":
    unittest.main()
