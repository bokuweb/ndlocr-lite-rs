# Page OCR library API

`recognize-page` is also available as a library call, so other crates (for example a
PDF → DOCX converter) can run page OCR without spawning the CLI.

```rust
use ndlocr_lite_rs::{PageOptions, recognize_page_gray_u8, recognize_page_rgb_u8};

let options = PageOptions {
    structure_rules: false,
    trace: true,
    ..PageOptions::default()
};
let page = recognize_page_gray_u8(&gray, width, height, &options)?;
println!("{}", page.body);
let trace_json: Vec<u8> = page.trace_json().unwrap(); // same bytes as --trace-json
```

## Contract

- `recognize_page_rgb_u8(rgb, width, height, &PageOptions) -> Result<PageResult>` takes
  row-major 8-bit RGB (`width * height * 3` bytes). It is the whole body of
  `recognize-page`: the CLI loads the image and the optional `--post-dict` /
  `--rule-pack` files, calls it, and writes the result.
- `recognize_page_gray_u8` takes 8-bit grayscale (`width * height` bytes), for example a
  DeviceGray image embedded in a PDF. Each value is copied to R, G and B, which is what
  the CLI does when it opens a grayscale PNG, so the result equals the CLI on that PNG.
- A buffer whose length does not match the dimensions is rejected before any work.
- `PageOptions::default()` equals the `recognize-page` CLI defaults (a test checks this).
  `RecognizePageArgs::page_options` converts CLI arguments.
- Models are loaded from the paths in `PageOptions` on every call, like the CLI.
- `PageResult` has `lines` (what `--output-docx` writes), `body` (`lines` joined by
  `\n`; what the CLI prints and `--output-txt` writes) and, when `PageOptions::trace` is
  set, `trace`: the `--trace-json` document (`schema_version`, `width`, `height`,
  `settings`, `stages.*`, `body`). `PageResult::trace_json()` serializes it with the
  same bytes the CLI writes.

## Compatibility check

The CLI output must not change. Compare stdout, `--output-txt`, `--trace-json` and
`--output-docx` from the previous binary and the new one on the same inputs (kanpo
pages, vertical pages, band-fallback pages, a missing detector model, `--detect-tiles 2
--quality-boost`, `--use-deim-detection false` with a rule pack and a post dictionary).

The frozen vertical fallback quality regression converts CLI arguments with
`page_options(None, None)` before calling the cached recognizer. It varies the
quality options on that same `PageOptions` so the test still detects skipped finishing.
