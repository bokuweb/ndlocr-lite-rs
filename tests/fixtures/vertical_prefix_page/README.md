# Page prefix recovery regression

`bold31.png` is the unchanged known development image
`uncertain_bold_31_pad0` from the N experiment. `case.json` records its original
PNG/truth SHA256 and fixed complete text. The corresponding original RGB,
glyph annotations, generation conditions and model provenance are preserved in
the vertical-partition-agreement benchmark history.

This known-font synthetic column is a regression fixture, not a new holdout.
It requires the pinned official DEIM and PARSeq30 models and charset. Font
files are not required to run it. The ONNX lib test fails if assets are missing;
it checks the actual Page API in eval/default modes and session reuse.

The e5 runtime drops the final scalar in this Page route; O recovers the
complete strict-prefix extension. Cross-platform fixed confidence bits or a
fixed DEIM bbox are not assumed.
