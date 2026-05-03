# Batch PARSEQ preprocessing performance spec

## Target

Phase 5 performance work for PARSEQ batch recognition.

## Required behavior

- The in-place PARSEQ preprocessor must produce exactly the same BGR NCHW tensor
  values as the allocating preprocessor.
- Batch recognition should write horizontal line crops directly into their final
  batch tensor slot, avoiding a per-line temporary tensor and copy where it is
  faster.
- Vertical line crops may keep the allocating path when it preserves better cache
  locality.
- Invalid input and output buffer sizes must be rejected.

## Measurement

- Use `cargo bench --bench parseq_batch_preprocess_bench`.
- Keep `cargo bench --bench parseq_preprocess_bench` available for single-line
  preprocessing regressions.
