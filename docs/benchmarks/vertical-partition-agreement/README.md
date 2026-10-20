# Vertical fragment prefix recovery

This packet records the strict-prefix recovery added after `e5b11b4` (O).
When a Fragment+Auto result reaches the selected PARSeq output's fixed T−1/T
scalar boundary, two different safe partitions of the original vertical RGB
must agree on a longer string that preserves the entire original raw prefix.
H retains priority. The public legacy splitter keeps its existing behavior;
the supported splitter is private to this additional recovery.

## Measured scope

| Comparison | Result |
| --- | --- |
| 414 original RGB inputs × 3 models × 8 API routes | 9 input/model pairs improved; no per-input regression |
| New N development images among those gains | 7, all from known-font synthetic text |
| Additional N2 images | 0 new gains |
| Model30 Fragment+Auto, none/width/strict | 21 fewer character errors and 9 fewer word errors |
| Model50 / Model100 | unchanged; no positive case invented |
| raw Session, normal pool, explicit Horizontal | 6,210 input/model/route pairs unchanged |
| Complete-text boundary controls | all54 unchanged |
| Empty diagnostic truth controls | all114 unchanged |
| Page API/CLI: 444 IDs, 443 unique RGB, 8 routes | 0 text changes; 0 regressions; line boxes/confidence bits unchanged |

The Page comparison uses the exact e5 runtime as its baseline. Its old293
baseline measurements were reused only after checking that the older captured
runtime differs from e5 through three `cfg(test)` declarations. The other151
baseline images and all444 candidate inputs were measured again. The original
`paragraph_raw` / `paragraph_normalized` controls have identical RGB and truth;
both IDs are retained and are not counted as independent images.

These are development controls, including known fonts, synthetic text and
shapes, and existing document crops. They are not an independent real-document
holdout or evidence that training did not contain similar fonts. Empty truth
for shape/margin controls is a diagnostic convention. Per-input changes are
reported; aggregate percentages on this mixed set would be misleading.

The seven new direct-Fragment gains still have errors in the detector-cropped
Page inputs. This patch does not claim Page accuracy gains. Follow-up work
will inspect the different crop geometry and actual recovery rejection reasons.

## Reproduce without models

Python3 with Pillow and a Rust toolchain are required. Run from any directory;
`--output-dir` must name a new directory:

```sh
python3 /path/to/vertical-partition-agreement/reproduce.py --output-dir /tmp/vertical-partition-check
```

The reader restores the frozen source into the output directory, builds its
ordinary lib test binary, and runs the actual Rust combine and geometry
recorders. It checks 4,432 literal greedy records, all3,190 Horizontal crop
bindings, 1,242 Rust decisions/plans, the saved native API routes, and all native
and Page official CSVs. A separate bit-vector edit distance verifies the error
counts and denominators. This operation loads no models, performs no recognition,
and needs no font files. Cargo may fetch its ordinary locked dependencies.

To replay against a matching checkout instead, pass
`--manifest-path /path/to/Cargo.toml`. The policy/source hashes must match.
Saved native records are measurements from the captured run; this model-free
reader verifies and re-scores them rather than claiming to rerun inference.

```sh
python3 -m unittest discover -s /path/to/vertical-partition-agreement -p 'test_*.py'
python3 /path/to/vertical-partition-agreement/history_reader.py
```

The history reader checks every content-addressed object, including aliases to
the original inputs. To inspect a selected earlier snapshot at its original
relative paths:

```sh
python3 /path/to/vertical-partition-agreement/history_reader.py --restore-prefix tmp/vertical-strict-prefix/native-source-before-green --output-dir /tmp/vertical-history
```

Use the default frozen overlay for a standalone replay. Historical snapshots
and commands preserve the context of their own experiment, including paths,
failures, intermediate assertions, and then-current status statements.

## Files and provenance

- `inputs.zip` / `inputs.json`: one copy of the414 PNG/RGB/truth inputs, glyph boxes and SHA metadata.
- `horizontal-inputs.zip`, `crop-bindings.json`: 1,653 distinct RGB/dimension tuples and3,190 exact original-pixel crop bindings.
- `raw-traces.jsonl.gz`: 1,242 Auto and3,190 Horizontal actual saved records. Reused results are not new recognition calls.
- `combine-manifest.json.gz`, `route-links.json.gz`, `expected-decisions.json.gz`: literal inputs, actual crop linkage and expected Rust outcomes.
- `native-records.json.gz`, `native-metrics/`: the414×3×8 API comparison. Baseline batches were measured as disjoint342 and72 groups; candidate batches contain all414.
- `page-inputs.json`, `page-inputs.zip`, `page-records.json.gz`, `page-metrics/`: full Page comparison with shared original payloads and exact one-LF CLI transport checks.
- `source-overlay.zip`, `source-overlay.json`: exact O Rust source with e5 project assets and the captured dependency-lock snapshot. No root Cargo.lock is tracked separately.
- `history.zip`, `history-index.json.gz`: deduplicated source/specification/freeze/command/failure evidence for K, L, M, N, N2 and O.

K was rejected before NN because cuts remained (116→36 events). L removed the
measured cuts but regressed18 input/model pairs after2,196 raw Horizontal calls,
so it was not applied globally. Initial M kept the whole prefix but gained only
on known inputs. N added a low-confidence one-scalar correction; its7 new gains
were all strict-prefix extensions, and N2 found no new correction gain. That
correction branch remains held. O retains the fixed M policy and its later7
strict-prefix gains, without deploying N's character replacement.

Model and charset hashes are preserved; no weights or system font binaries
are included. Native executables remain in local lossless archives and are not
shipped here. The older document provenance is also recorded in the repository's
`real-vertical-scan` and `vertical-capacity` benchmark packets.

Additional recognition reuses the borrowed Session. Raw API calls, outer batch
calls, Page calls, CLI processes, loads, and internal forwards are different
counts. Uninstrumented forwards and loads are null. Concurrent elapsed times
are not a speed comparison.
