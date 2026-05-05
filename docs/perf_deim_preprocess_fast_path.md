# Faster DEIM preprocessing

## Problem

DEIM preprocessing pads each page to a virtual square, nearest-resizes it to the
model input size, and normalizes RGB pixels to NCHW floats. The current direct
implementation avoids the large padded intermediate image, but it still repeats
coordinate division and normalization arithmetic for every output pixel.

## Behavior

The optimized path must preserve the existing tensor values and metadata:

- invalid input validation remains unchanged,
- `padded_wh` remains `max(width, height)`,
- sampled source coordinates remain `dst * padded_wh / input_len`,
- padding pixels continue to normalize as black pixels.

## Benchmarks

Run:

```sh
cargo bench --bench deim_preprocess_bench -- --noplot
```

The benchmark compares a checked legacy-direct implementation against the
current production path for common DEIM input sizes.

Local result for this change:

| Case | legacy direct | current | Delta |
| --- | ---: | ---: | ---: |
| `595x842 -> 800x800` | 1.2083 ms | 978.68 us | 19.0% faster |
| `1240x1754 -> 800x800` | 1.7647 ms | 1.1676 ms | 33.8% faster |
| `1754x1240 -> 800x800` | 1.7756 ms | 1.2130 ms | 31.7% faster |
