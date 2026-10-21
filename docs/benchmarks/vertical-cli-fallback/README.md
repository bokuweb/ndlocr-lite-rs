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
