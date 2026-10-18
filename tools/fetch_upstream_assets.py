#!/usr/bin/env python3
"""公式の固定リビジョンからモデル・文字集合・通知をハッシュ検証して取得する。"""
import argparse
import hashlib
import json
import os
import tempfile
import urllib.request
from pathlib import Path


def fetch_asset(asset, directory, offline=False):
    """検証済みファイルだけを公開し、失敗時は既存ファイルを保持する。"""
    directory = Path(directory)
    name = asset["name"]
    if Path(name).name != name or name in ("", ".", ".."):
        raise ValueError("asset name must be a filename")
    target = directory / name
    expected = asset["sha256"]
    if target.is_file() and hashlib.sha256(target.read_bytes()).hexdigest() == expected:
        return
    if offline:
        raise ValueError(f"missing or corrupt asset: {name}")
    directory.mkdir(parents=True, exist_ok=True)
    temporary = None
    try:
        with urllib.request.urlopen(asset["url"], timeout=60) as response:
            with tempfile.NamedTemporaryFile(dir=directory, delete=False) as output:
                temporary = Path(output.name)
                digest = hashlib.sha256()
                while block := response.read(1024 * 1024):
                    digest.update(block)
                    output.write(block)
        if digest.hexdigest() != expected:
            raise ValueError(f"checksum mismatch: {name}")
        os.replace(temporary, target)
    finally:
        if temporary is not None:
            temporary.unlink(missing_ok=True)


def main():
    """マニフェスト全体を取得し、オフライン時は再検証だけを実行する。"""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("directory", type=Path)
    parser.add_argument("--offline", action="store_true")
    args = parser.parse_args()
    manifest = json.loads((Path(__file__).resolve().parents[1] / "docs/upstream-assets.json").read_text())
    for asset in manifest["assets"]:
        fetch_asset(asset, args.directory, args.offline)
        print(f"verified {asset['name']}")


if __name__ == "__main__":
    main()
