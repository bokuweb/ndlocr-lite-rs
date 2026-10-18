# Deterministic CPU thread configuration tests

The ONNX unit suite reproduced a race in `env_override_clamps_to_sane_range`:
other tests concurrently remove `NDLOCR_INTRA_THREADS`. Rust tests are parallel,
and environment mutation is process-wide. Before changing code, the failing log
is recorded in the local validation results. Extract a pure resolver accepting
the optional override, CPU count and pool count. Keep the public environment
lookup and the existing clamps (override 1..32, automatic 1..8) unchanged.
Test zero/large pools, invalid overrides and fixed CPU counts without mutating
the process environment. This is a test-isolation fix, not a speed claim.
