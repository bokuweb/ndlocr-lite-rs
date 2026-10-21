# Additional real controls fixed before recognition

`controls-before-nn-v1.zip` preserves the original public NDL downloads, source and
HTTP receipts, exact crop/GT freezes, and the two-pass visual-review notes.
`index.json` gives SHA-256 and size for the archive and every member. Paths inside
are relative to the repository root; `workspace_root` records the original checkout.
No model, font, or native executable is included.

The accepted set is **two books / six physical column images / eleven scoring IDs**:
Ichiyo PID1088069 and Jigokuhen PID1908126. Ambiguous Unicode forms were enumerated
before recognition, and all forms must be nonregressing. Ruby stays in the image;
GT measures main text. These are assistant transcriptions reviewed twice by the
same assistant, not independent expert labels or a claim of training disjointness.
See [the source-selection record](../../real_vertical_additional_control_sources.md).
Lemon and Haru remain unscored because of overlapping slanted columns and faded
characters respectively. No guessed GT was added.

**New-set OCR calls and model loads are zero.** Its production baseline has not yet
been chosen; the separate FVB full matrix must be resolved first. This packet is
not a new accuracy result and does not alter the frozen FVB inputs or gates.

The archive also preserves a separate NN-free diagnosis of the existing public Q/S
results: 432 raw input/model pairs, with five near-capacity, aspect-guard-rejected
pairs. Four body pairs already contain substitutions; the fifth is a nonletter
control. This does not support adopting a relaxed guard, and does not prove such a
change could never help. It is not a direct runtime trace of H adoption or T matching.
The original Q/S evidence and reader are in `../vertical-low-stop-development/`.
