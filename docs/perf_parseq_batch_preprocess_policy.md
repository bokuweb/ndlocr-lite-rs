# PARSEQ batch preprocess policy

## Problem

`ParseqPool` builds a `[N, 3, H, W]` tensor before each batch inference. The
current implementation can either write each crop directly into its final batch
slot, or allocate a per-crop tensor and copy it into the batch buffer. The
previous heuristic used direct slot writes for large vertical batches, but fresh
local benchmarks show that the allocating path is faster for that shape.

## Behavior

The policy should keep choosing by measured crop shape:

- small batches use direct slot writes to avoid per-crop allocation,
- large batches use the allocating path when it benchmarks faster than direct
  writes.

The policy is an implementation detail of the cached ONNX path; it must not
change tensor values or public recognition behavior.

## Benchmark

Use:

```console
cargo bench --bench parseq_batch_preprocess_bench -- --noplot
```

Report same-machine before/after benchmark deltas in the PR body.
