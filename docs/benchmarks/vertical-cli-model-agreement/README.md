# C1 frozen before recognition

C1 rechecks an undercovered, image-supported vertical CLI crop with the existing
30/50 pools. It requires exact model agreement, the complete original prefix and
whitespace sequence, image-anchor coverage, and finite confidence >=0.98 above
both the base and user minimum. The original crop and downstream processing stay
unchanged. Only cascade-on initial100 with the three standard actual pool shapes
is eligible. This is isolated from production FVB and the separate FVC experiment.

`pure-before-nn-v1.zip/json` binds 926 members: both sources, fresh build receipts
and executable hashes, pure Red/Green/property tests, all 96 PNG/RGB/GT controls,
generation receipts, and failed attempts. Native executables, fonts and weights
are excluded. **At packet creation, C1 recognition/model loads were zero.** The
known real-CLI regression and all accuracy gates remain pending at this stage.

The controls comprise 48 varied text columns, 12 repeated columns, 12 portrait
horizontal controls and 24 nonletters. Independent masks verified 4,860 glyphs.
`known-inputs-before-nn-v1.json.gz/index.json` separately freezes the 1,143 known
physical headers and their source manifest hashes before C1 recognition; no new
PNG/RGB hash overlaps. These are known-font development images, not unseen-font
or training-disjoint data.

The pure suite passed eight tests, including 128 property cases; the candidate's
full default library passed 448 with 38 ignored after missing runtime fixtures
were supplied. The original 32 missing-file failures, shrinking seeds and fixture
hash map are preserved. ONNX lib Clippy passed with the existing 18 warnings.
The three own artifacts for each variant were fresh=false and the CLI hashes
differ. Production runtime is unchanged.

```sh
python3 tools/verify_vertical_cli_model_agreement.py
python3 -m unittest discover -s tests -p test_vertical_cli_model_agreement_packet.py
```

The reader requires Pillow, and checks every member, fresh build binding, RGB,
glyph boxes, and novelty registry without models or fonts. The archived manifest
records original absolute paths; archive images live in `controls/inputs/`.
Full Rust suite replay also needs the 561 fixtures from root commit f99e1fe;
their hashes are recorded. The compile-time PNG and new CLI fixture are embedded.
See [the protocol](../../vertical_cli_model_agreement_proposal.md). This packet
proves preparation and pure behavior, not an accuracy gain or adoption.
