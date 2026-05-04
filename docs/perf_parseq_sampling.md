# PARSEQ sampling performance spec

## Target

Phase 5 performance work for single-line PARSEQ preprocessing.

## Required behavior

- Sampling changes must preserve the exact BGR NCHW tensor values produced by
  `preprocess_rgb_u8`.
- Horizontal and vertical line crops must both keep the same resize and rotation
  semantics.
- The hot pixel loop should avoid repeated coordinate mapping work and avoid
  per-pixel rotation branches where possible.

## Measurement

- Use `cargo bench --bench parseq_preprocess_bench`.
- Keep `cargo bench --bench parseq_batch_preprocess_bench` for batch-path checks.
