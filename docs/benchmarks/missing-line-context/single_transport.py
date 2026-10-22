"""Separate the CLI println terminator from literal recognition text."""
def recognition_content(stdout: bytes) -> bytes:
    if not stdout.endswith(b'\n') or stdout.count(b'\n') != 1:
        raise ValueError('expected one println LF terminator and no embedded LF')
    return stdout[:-1]
