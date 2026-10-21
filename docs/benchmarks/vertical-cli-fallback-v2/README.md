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

With Pillow installed, run:

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
The new72 default accuracy comparison is a separate in-progress run and is not an adoption result.

Its first attempt stopped with Python exit120 during disk exhaustion, leaving 129 successful baseline command receipts and no complete gate.
stderr was empty, so the original exception is unavailable. All partial outputs remain private and are excluded from adoption.
After lossless archival of closed experiment RGBs and a fresh check of every original binding, all 576 results are being remeasured in a new directory.
Source, models, GT, images and executables are unchanged; no partial results are reused.

`options-plan-before-nn-v1.zip` freezes 384 explicit argv for eight inputs and six nondefault profiles (1,152 planned metric rows).
The model-free planner rejects incomplete/duplicate/unknown input sets and replays every argv with `--options-plan`.
It runs zero recognition; execution requires the completed new72 nonregression/positive-gain gate first.

`feature-disabled-v1.zip` records a newly compiled v2 default CLI and the restored exact FVB default CLI.
All 16 missing/dummy-model × body/white × DEIM off/on × variant processes preserve exit/stdout/stderr/body bytes.
No model loads or NN calls occur. The reader was added from missing-function Red to all three packet tests Green; replay with `--disabled`.
