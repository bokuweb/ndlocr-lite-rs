# Batched line recognition for `recognize-page`

## Problem

`recognize-page` already keeps PARSeq sessions cached, but it still prepares and
runs recognition one line at a time in the CLI path. That leaves existing
batched pool APIs unused and pays per-line dispatch overhead even when a page
contains many line detections.

## Behavior

Before PARSeq recognition, the command should prepare valid line crops in the
same order as the filtered detections:

- ignore invalid or out-of-image boxes,
- apply `--line-crop-padding` with image-bound clamping,
- preserve bbox, confidence, vertical flag, and `pred_char_count` metadata.

When the `onnx` feature is enabled, initial recognition should run in batches:

- if cascade is disabled, batch all crops through the 100-character model,
- if cascade is enabled, bucket crops by `pred_char_count` and batch each model,
- keep the existing 30-to-50 and 50-to-100 fallback thresholds,
- keep long-line split and quality boost behavior after the initial result.

The output order, confidence filtering, post dictionary, structural rules, and
rule-pack application must remain unchanged.

Line crop preparation should avoid per-pixel copy overhead. Once a crop box is
validated, each output scanline can be copied from the source RGB buffer as one
contiguous slice.

## Expected effect

Pages with many lines should spend less time in PARSeq recognition by reducing
per-line session calls and by allowing the existing `ParseqPool` batch paths to
group preprocessing and inference work. Scanline crop copies should also reduce
CPU overhead before recognition starts.
