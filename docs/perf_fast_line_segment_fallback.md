# Fast line segmentation fallback for `recognize-page`

## Problem

When DEIM detection is disabled or fails, `recognize-page` falls back to simple
RGB threshold line segmentation. The existing fast implementation can be faster
on large scanned pages, but benchmark results show it is not always faster on
smaller pages.

## Behavior

The fallback should choose the segmentation implementation by page size:

- small pages keep the naive implementation to avoid regression,
- large pages use the fast implementation,
- both implementations must keep returning identical boxes for covered cases.

The CLI warning should describe this as the band fallback rather than implying
that the slow naive path is always used.

## Benchmarks

Run:

```sh
cargo bench --bench line_segment_bench -- --noplot
```

On the local benchmark run used for this change:

- `595x842`: naive 629.18 us, fast 1.0476 ms, so small pages should keep naive.
- `1240x1754`: naive 3.5823 ms, fast 3.3542 ms, so large pages improve by
  about 6.4% with fast.
