# FVC pure stage and new controls (no OCR yet)

FVC proposes extending FVB's supported single-column direction fallback to the CLI's
existing no-detection/bands branch. Production is unchanged. **CLI orchestration is
not connected, no actual model was loaded, and no accuracy gate has passed.**

`pure-before-nn-v1.zip` contains the prior specs, Red/Green source snapshots and logs,
the isolated green source, generation receipts, 72 fixed PNG/RGB/GT controls, and
actual crop records. `index.json` binds every member and the outer archive by size
and SHA-256. Models, fonts, native executables and root Cargo.lock are excluded.
A pinned Cargo.lock remains inside the isolated source archives for reproducibility.
Paths are repository relative; absolute manifest paths refer to the recorded
`workspace_root` and must be relocated before replay elsewhere.

The pure stage covers padding/clamping, original RGB and bbox consistency, no extra
recognizer on detected/unsupported input, failed or low-confidence attempts, and
vertical model buckets including equality/custom thresholds. Red failures preceded
the minimal implementations. A 128-case property test checks translated colored
glyphs and padding. The full default library suite passed **447 tests / 39 ignored**.
The subsequent geometry-only recorder adds one explicit ignored diagnostic test.

The new controls comprise 36 diverse body columns, 12 repeated columns, 12 portrait
horizontal row groups and 12 nonletter patterns. Their 4,104 vertical glyphs were
reconstructed independently; PNG/RGB bytes match and none duplicates the known 1,071
physical inputs. They use known fonts and are synthetic development controls.

Before NN, 72 inputs × raw/prepared × three padding settings produced **432 geometry
records / 282 selected crops**. All callback RGB crops match independent Pillow crops;
20,142 annotated glyph-bound checks found zero cuts. The selected counts include
48 nonletter cases under the dummy callback; actual output rejection and accuracy
remain unmeasured. All actual background-normalization outputs were unchanged.
Zero padding preserved the old Page helper result. Crop success is not accuracy evidence.

Real-model CLI work must wait for the separate FVB full gate to be resolved, then
follow the precommitted new-control and complete-matrix gates in
[the proposal](../../vertical_cli_fallback_proposal.md).

## Subsequent isolated CLI adapter, still before model recognition

`adapter-before-nn-v1.zip` and `adapter-before-nn-v1.json` preserve the next stage.
The isolated CLI now calls the prepared-crop adapter and reuses an accepted result;
production remains unchanged. Two prepared-crop tests failed before implementation,
and the reuse test failed before its branch was implemented. All eight CLI adapter
unit tests are green; the full default library passed 451 tests / 40 ignored, and
21 existing app-layout/argument/feature-disabled integration tests passed.

Both ONNX CLI executables and the identical regression-test source were compiled
with fresh own-crate artifacts and frozen separately by SHA. **Actual ONNX CLI
Red/Green and new-control accuracy measurements have not run.** The pinned known
regression is a single model30 override with cascade off and DEIM off/on, checking
literal body and stdout's one framing LF. It does not prove default100/cascade behavior.
Use `NDLOCR_TEST_CLI_BINARY` to select each frozen executable explicitly.

The complete no-feature CLI was exercised separately: body/white × DEIM off/on ×
missing/dummy assets × both sources = 16 processes. Exit, stdout, stderr and body
bytes/existence matched exactly. The earlier eight-process attempt also had equal
product outputs, but its test incorrectly expected the feature error before the
missing-file error. That failed expectation and all original outputs are retained;
no production behavior was changed to satisfy it. The five-byte `dummy.onnx` in this
packet is a placeholder, not model weights. ONNX model calls/session loads remain zero.

The two final source archives, source bindings, build logs, pure Red/Green, and
no-feature CLI command/output records are included. Model weights, native executables,
fonts, and duplicate copies of the original 72 controls are excluded. The prior
control index is bound by SHA. Cached-recognizer methods, the feature-disabled
recognition block, and downstream context/dict/rule-pack/output code remain byte
identical to frozen FVB; only the isolated orchestration and adapter changed.
