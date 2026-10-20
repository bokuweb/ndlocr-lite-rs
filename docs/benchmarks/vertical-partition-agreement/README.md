# Vertical fragment prefix recovery

**Page measurement correction (2026-10-21):** the original candidate CLI/Page binaries were identical to the baseline binaries. That “444 IDs unchanged” conclusion is withdrawn and retained in `withdrawn-page-v1/`. A fresh build of73e9ab4 now shows18 Page input/route changes across9 synthetic development images, with no per-input regression. Distinct artifact SHA values, rebuild receipts and a known positive Page preflight are checked explicitly.

This packet records the strict-prefix recovery added after `e5b11b4` (O).
When a Fragment+Auto result reaches the selected PARSeq output's fixed T−1/T
scalar boundary, two different image-supported partitions of the original vertical RGB
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
| Page API/CLI: 444 IDs, 443 unique RGB, 8 routes | 9 images improve in model30 eval/default:18 input/route pairs; 0 per-input regressions |
| Page model30, each eval/default route, none/width/strict | 13 fewer character errors and9 fewer word errors |
| Page model50/model100 and both CLI routes | unchanged |
| Page metadata | all line counts/boxes unchanged; confidence bits change in the same18 pairs |

The Page baseline is the exact e5 runtime. Its old293 measurements were
reused after checking three `cfg(test)` declaration differences. The other151
baseline inputs were measured with source that differs from e5 only by a test
module declaration. Both fresh candidate artifacts have `fresh=false` build
receipts and SHA values distinct from the baseline. All444 candidate inputs
were remeasured on8 routes. Official scoring and a separate bit-vector distance
agree on all21,312 character/word error and denominator rows. The original
`paragraph_raw` / `paragraph_normalized` controls have identical RGB and truth;
both IDs remain and are not counted as independent images.

The9 Page gains are2 earlier `long_vertical` controls,6 N controls and1 N2
control; all are known-font synthetic images. Direct native gains also occur
only in synthetic development controls. O adds no independent real-document
gain claim. H's earlier real-document results are recorded in its separate
benchmark packet.

These are development controls, including known fonts, synthetic text and
shapes, and existing document crops. They are not an independent real-document
holdout or evidence that training did not contain similar fonts. Empty truth
for shape/margin controls is a diagnostic convention. Per-input changes are
reported; aggregate percentages on this mixed set would be misleading.

A separate diagnostic executable matches all864 fresh Page API signatures.
Its16,806 actual primary crops and928 O part crops match independent original
pixel crops. The98 annotated successful plans have no interior glyph-BBox cuts;
19 unannotated plans remain separate. Physical detector-crop ink loss and raw
errors without ink loss both occur. Another111 annotated case/route occurrences
fall back to Horizontal recognition of101..146 small bands, bypassing vertical
recovery; this is a next measure, not a policy changed in O.

A new mandatory-asset ONNX lib regression also exercises the actual Page path:
e5 fails on a missing final scalar, while O passes eval/default, invalid-input
handling and reuse. Only this test module and fixture were added after the
measured73e9ab4 runtime; production inference code remains the measured O.

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
`--manifest-path /path/to/Cargo.toml`. The policy/source hashes must match the measured73e9ab4 source, including its test declarations; the current checkout adds a later Page regression. Prefer the default frozen overlay.
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
- `page-revalidation-*.zip`: exact additional trace, input, source and command payloads, split into bounded lossless volumes. `history-index-before-page-v2.json.gz` retains the earlier path mapping.
- `page-artifact-binding.json`: fresh candidate/baseline binary identities and the measured275-file Rust source mapping.
- `withdrawn-page-v1/`: the incorrect old candidate results, metrics and reports, retained as failed validation evidence.

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
