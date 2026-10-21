# FVC v2: preserve the existing finishing pipeline

The isolated v1 adapter reused an accepted initial recognition without running the existing short-line/split/quality/dense finishing loop.
v2 extracts that unchanged loop and applies it once to the accepted initial result; legacy recognition still finishes once as before.
Production remains FVB. The running v1 full matrix is evidence for its frozen v1 source, and cannot establish the v2 gate.

`before-nn-v1.zip` / `before-nn-v1.index.json` freeze 72 new images before recognition, 4,392 glyph annotations,
the 1,239 known input headers, both source trees, failing/passing pure tests and fresh compilation receipts.
Baseline uses the unchanged FVB binary already frozen with a fresh build receipt; v2 CLI/test were compiled anew.
Default tests: 453 passed / 40 ignored. Two finishing callback assertions failed before the minimal fix, then all ten adapter tests passed.
Compile-only missing-fixture and ENOSPC failures remain in the packet. No model ran during those builds.

The first archive transport used absolute input member paths and was rejected by the reader. Before recognition, only archive member names were normalized;
PNG/RGB/GT/source bytes stayed unchanged, and the original archive hash is retained in the index. The original container remains private.
Input path strings inside manifests still document the actual original paths.

With Python 3.12+ and Pillow installed, run:

```sh
python tools/verify_vertical_cli_fallback_v2.py
```

This authenticates all members, fresh build events, unchanged initial batch/finishing loop, all PNG/RGB bytes and glyph bounding boxes,
and zero PNG/RGB overlap with the 1,239 known input headers. It does not rerender Mac fonts or run recognition.
Independent rerendering happened during generation; that receipt is retained separately from these model-free checks.
Before-NN protocols: `docs/vertical_cli_fallback_postprocessing.md` and `docs/vertical_cli_fallback_v2_controls.md`.

`known-finish-v1.zip` retains four known-setting test attempts and the mandatory literal CLI check.
The charset setup failure and two unsuitable fixtures remain failed; all nine recorded setting results still match the existing full pipeline.
The 12-character fixture passes all three settings and actually changes confidence with `quality_boost_min_delta=-100`,
so omitting the finishing stage would fail that case. The ordinary delta0.03 cases remain unchanged.
All nine captured crop RGBs are independently checked against the input PNG/bbox. No GT is passed to these recognitions.
The literal CLI check passes DEIM off/on with body/stdout assertions; its temporary body files are not separately retained.
These are known regression checks, counted as zero new accuracy gains. Replay with `--known`.
The new72 default comparison completed with 576 CLI results / 1,728 independently verified metric rows,
zero per-input character/word regressions, and 22 raw gain pairs on ten body images. This is a preliminary gate, not an adoption result.

Its first attempt stopped with Python exit120 during disk exhaustion, leaving 129 successful baseline command receipts and no complete gate.
stderr was empty, so the original exception is unavailable. Partial outputs are retained in `new72-cli-matrix-v1.zip` and excluded from adoption.
After lossless archival of closed experiment RGBs and a fresh check of every original binding, all 576 results were remeasured in a new directory.
Source, models, GT, images and executables are unchanged; no partial results are reused.

`options-plan-before-nn-v1.zip` retains the initial 384-argv plan, whose quality-boost boolean value syntax was rejected before model loading.
`options-plan-before-nn-v2.zip` corrects only the flag format and output directory, preserving all eight inputs/six profiles (1,152 planned metric rows).
The actual Rust CLI parser checks all 384 argv; the compiled baseline/candidate CLIs also pass 48 missing-image setting vectors without loading models.
The model-free planner rejects incomplete/duplicate/unknown input sets and replays every argv with `--options-plan`.
It runs zero recognition; execution requires the completed new72 nonregression/positive-gain gate first.

`feature-disabled-v1.zip` records a newly compiled v2 default CLI and the restored exact FVB default CLI.
All 16 missing/dummy-model × body/white × DEIM off/on × variant processes preserve exit/stdout/stderr/body bytes.
No model loads or NN calls occur. The reader was added from missing-function Red to all three packet tests Green; replay with `--disabled`.

`QA-and-full-scope-v1.zip` retains the frozen ONNX lib run (465 passed / 115 ignored), successful all-targets Clippy logs,
and the new complete-scope union's Red/Green proof. No warnings occur in the new FVC app/finish-test files;
the frozen old FVB iterator/test-identity warnings remain in the logs and are already addressed by the equivalent root32ef cleanup.
The proposed v2 full gate has 1,311 physical / 1,317 scoring IDs, 26,220 image results and 79,020 metric rows.
The new72 gate is passed. The complete v2 runner still needs implementation/dispatch and full nonregression validation.

`new72-cli-matrix-v1.zip` contains the complete run and the excluded 129-receipt attempt. Replay all 1,728 scores/864 comparisons with `--new72`.
`option-parser-preflight-v2.zip` preserves the actual parser Red/Green and all native preflight logs, including the first validator wording failure.
Replay that proof with `--parser-preflight`. The nondefault image matrix uses the corrected v2 plan and remains separate from these NN-free checks.
