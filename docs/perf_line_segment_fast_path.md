# Line segmentation fast path performance spec

## Target

Phase 5 performance work for hot paths used while recognizing a page.

## Required behavior

- The fallback recognizer must not be moved to a slower segmentation implementation.
- Fast segmentation must return the same bounding boxes as the naive implementation
  for existing fixtures and synthetic page-like inputs.
- PARSEQ preprocessing must keep the same BGR NCHW tensor values.
- Optimization must stay generic structural cleanup only; no document-type-specific
  OCR corrections belong in line segmentation.

## Measurement

- Use `cargo bench --bench line_segment_bench` before and after changes.
- Use `cargo bench --bench parseq_preprocess_bench` for PARSEQ preprocessing changes.
- Report the benchmark deltas in the PR body.
