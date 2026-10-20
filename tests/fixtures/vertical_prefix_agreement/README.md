# Fixed-capacity vertical prefix agreement

Three original-RGB PNGs from the fixed M/N development controls. The first two
were generated after the strict M policy was frozen, with 31 and 35 Unicode
scalars, font size 48 px, 52 px advance, 56 px column width and zero side padding.
The third has a one-scalar disagreement in the original raw prefix: even when
two partitions agree on a replacement, strict prefix recovery must reject it.
`truth` is diagnostic and is never passed to recognition or recovery.

The public ignored ONNX lib test loads the pinned official PARSeq30 and charset;
the existing inference CI runs it with `--include-ignored`. It asserts literal
text, original RGB preservation, confidence validity and reuse after invalid
input. Cross-platform floating-point confidence bits are not pinned.
No font files, model weights, or machine paths are included here.
