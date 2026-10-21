"""Freeze actual Rust sources, fixtures and literal compiler include resources."""
import hashlib
import json
from pathlib import Path
import re


def source_files(root):
    root=Path(root).resolve();paths={root/'Cargo.toml',root/'Cargo.lock'}
    for folder in ['src','tests','examples','benches']:
        for path in (root/folder).rglob('*'):
            if path.is_file() and (path.suffix=='.rs' or path.is_relative_to(root/'tests/fixtures')):paths.add(path)
    macro=re.compile(r'\binclude(?:_bytes|_str)?!\s*\(')
    literal=re.compile(r'\s*("(?:[^"\\]|\\.)*")\s*,?\s*\)')
    for path in list(paths):
        if path.suffix!='.rs':continue
        text=path.read_text()
        for occurrence in macro.finditer(text):
            match=literal.match(text,occurrence.end())
            if not match:raise ValueError(f'unsupported dynamic include in {path}')
            target=(path.parent/json.loads(match[1])).resolve()
            if not target.is_relative_to(root):raise ValueError(f'include outside source root: {path}')
            if not target.is_file():raise FileNotFoundError(target)
            paths.add(target)
    result={}
    for path in sorted(paths):
        resolved=path.resolve()
        if not resolved.is_relative_to(root):raise ValueError(f'source outside root: {path}')
        result[str(path.relative_to(root))]=hashlib.sha256(path.read_bytes()).hexdigest()
    return result
