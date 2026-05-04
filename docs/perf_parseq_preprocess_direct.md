# Direct PARSEQ preprocessing into batch buffers

## Problem

The cached PARSEQ batch path allocates one temporary `Vec<f32>` per line during
preprocessing, then copies that tensor into the final `[N, 3, H, W]` batch
buffer. On pages with many line crops, this adds avoidable allocation and copy
work before ONNX Runtime inference starts.

## Behavior

PARSEQ preprocessing should expose an API that writes directly into a caller
provided `&mut [f32]` slot:

- output must be exactly `3 * input_width * input_height`,
- invalid RGB input length and invalid resize dimensions remain errors,
- output must match the existing allocating `preprocess_rgb_u8` result exactly,
- reusable scratch storage may cache horizontal and vertical sampling
  coordinates across calls with the same output size.

The public allocating API remains available and delegates to the direct writer.

## Expected effect

Batch recognition can fill the final tensor buffer without per-line tensor
allocation or an extra copy when the direct path is beneficial. Precomputed
sampling coordinates also reduce repeated resize math in common fixed-size
PARSEQ model inputs.

Benchmarks should compare both serial and rayon-parallel batch preparation. The
cached ONNX path uses a hybrid strategy: direct slots for small batches and
vertical crops, while keeping the allocating path for large horizontal batches
where the local tensor plus memcpy path benchmarks faster.
