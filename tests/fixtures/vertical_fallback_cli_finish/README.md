# Known finish regression

`arial12.png` is the exact `fva_vertical_arial_40_12_pad0` image already used by the
frozen FVC v2 known-option regression. Its SHA256 is
`8cbe07119044e486088c21b0c047dba59bacd5cb3fe1c30c8ce8eba913daaa34`.

This is a known development fixture, not a new accuracy control. It distinguishes
the initial recognition from the existing finishing pipeline by forcing the
quality delta to -100. That value is only a regression probe. The test compares
literal text and f32 confidence bits against the existing full batch pipeline.

`case.json` preserves the original public packet's provenance metadata, including
its historical local path; the regression reads the PNG in this directory.
