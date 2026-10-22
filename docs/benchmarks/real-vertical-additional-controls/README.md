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

**At this input packet’s creation, new-set OCR calls and model loads were zero.**
The original protocol and manifest retain that historical state.

After the complete FVB gate and all five e559 CI jobs succeeded, the separate
`FVB-baseline-real6-v1.zip` recorded 48 image results and 264 verified metric rows.
Its `.index.json` authenticates every member and records the scope of source binding.
This is a single production-baseline observation, not an adoption comparison.
See [results](../../real_vertical_additional_results.md). The original inputs/GT
remain unchanged. Replay without models or fonts:

```sh
python3 tools/verify_additional_real_baseline.py
python3 -m unittest discover -s tests -p 'test_additional_real_*.py'
```

The archive also preserves a separate NN-free diagnosis of the existing public Q/S
results: 432 raw input/model pairs, with five near-capacity, aspect-guard-rejected
pairs. Four body pairs already contain substitutions; the fifth is a nonletter
control. This does not support adopting a relaxed guard, and does not prove such a
change could never help. It is not a direct runtime trace of H adoption or T matching.
The original Q/S evidence and reader are in `../vertical-low-stop-development/`.
