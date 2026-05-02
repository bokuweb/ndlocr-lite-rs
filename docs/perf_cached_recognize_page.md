# Cached PARSeq sessions for `recognize-page`

## Problem

`recognize-page` used `parseq::recognize_rgb_u8_with_score()` for each line and
for each quality re-scoring candidate. That helper creates a new ONNX Runtime
session every time it is called. On pages with many lines, wall-clock time is
dominated by repeated model load / graph optimization rather than inference.

## Change

When the `onnx` feature is enabled, `recognize-page` should load PARSeq models
once per command invocation and reuse cached `ParseqPool` sessions for:

- the initial 30/50/100 cascade decision,
- long-line split re-recognition,
- quality-boost re-scoring candidates.

The non-ONNX build keeps the existing fallback path so tests and error messages
remain available without model files.

## Expected effect

The first page still pays model-load cost once per model, but subsequent line
recognition avoids repeated `Session::commit_from_file()` calls.

`NDLOCR_PARSEQ_PARALLELISM` can be used to raise the number of sessions per
model when processing larger pages. The default is `1` because short one-page
CLI runs are otherwise dominated by extra session load time.
