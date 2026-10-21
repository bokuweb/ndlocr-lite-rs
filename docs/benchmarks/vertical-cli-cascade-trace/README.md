# Frozen known-input CLI stage observation

`bundle.json` authenticates three known synthetic PNG/GT inputs and two frozen
sources: O/1f240be and the same source with an isolated observer. Models, fonts and
native executables are excluded. Production inference is unchanged.

The observer wraps each existing cached batch once, records original RGB and
result bits, and takes separate post-initial and final snapshots. The original
batch method is byte-identical. Pool batches are not internal forward counts.
Linux run39552644890 completed all 18 processes: every normal/observer output
matched, 27 crop instances matched the original PNGs, and all 54 metric rows
were independently verified. The first compile-fixture failure is retained.
This packet does not assert an accuracy improvement.

The Linux workflow builds fresh release executables and requires different CLI
hashes; the unused Page example may be identical. It runs three configurations
(default, forced single30, forced single50) on three images with both sources:
18 CLI processes, 54 official metric rows, and independent RGB/metric checks.
Every normal/observer stdout, stderr and body must match. Missing models fail.
Raw build/command/trace records survive failure. The source and inputs stay frozen.

```sh
python3 tools/cli_cascade_trace.py --validate
python3 -m unittest discover -s tests -p test_cli_cascade_trace.py
# Linux, pinned models already present:
python3 tools/cli_cascade_trace.py --output NEW_DIRECTORY
```

See [the protocol](../../vertical_cli_cascade_trace.md). Any policy change needs
its own specification, tests, new controls and complete regression gate.

`complete-linux-trace-v1.zip/json` preserves all 164 raw artifact members with
SHA-256 bindings. [Results](../../vertical_cli_cascade_trace_results.md) show
that all three defaults selected initial100, with no nonempty promotions.
Replay the completed evidence without models or fonts:

```sh
python3 tools/verify_cli_cascade_trace.py
python3 -m unittest discover -s tests -p test_cli_cascade_trace_packet.py
```
