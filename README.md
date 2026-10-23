# ndlocr-lite-rs

[![CI](https://github.com/bokuweb/ndlocr-lite-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/bokuweb/ndlocr-lite-rs/actions/workflows/ci.yml)

PoC repository for reimplementing the `ndlocr` OCR pipeline in Rust + ONNX.

## Capabilities

- Produce OCR output (JSON / XML / TXT)
- Reading-order sorting (horizontal and vertical text mixed)
- Run PARSeq / DEIM when the ONNX feature is enabled
- NDLOCR-Lite v1.2-compatible PARSeq cascade models (24px height, 30→50→100)
- Page OCR with DEIM line detection (using `pred_char_count`) + PARSeq
- `--line-crop-padding` to expand bboxes before line crops (try when scans clip tightly at the box)
- Optional `--quality-boost=true` candidate re-scoring for accuracy-focused runs
- Example that performs PDF → image conversion on the example side
- Benchmarks with `criterion` (preprocess, postprocess, line extraction)

## Horizontal multicolumn accuracy (2026-10-23)

Image-supported column separation, duplicate detection selection and reading order
improve horizontal Japanese Gazette OCR at the default DEIM threshold of **0.25**.
On an independent two-column sample (1,069 reference characters, 80 physical lines),
the implementation and manually transcribed reference were frozen before recognition:

| Engine | Character errors | CER |
|---|---:|---:|
| ndlocr-lite-rs | **1** | **0.094%** |
| Tesseract.js 6.0.1 | 36 | 3.368% |

Both engines produced identical raw text across three runs each. ndlocr-lite-rs had
one substitution, zero insertions/deletions, correct notice order and no excess
duplicate reference lines. The development full-page sample (2,613 characters)
scored 0.421% CER versus Tesseract.js's 22.465%.

The comparison uses identical PNGs, fixed DEIM/PARSeq models versus fixed
`tessdata_fast` Japanese (OEM 1, PSM 3, DPI 300), with NFKC and whitespace removal
only for scoring. NDLOCR quality boost, structure rules, dictionaries and rule packs
were disabled. This result covers the evaluated horizontal Gazette samples;
broader document accuracy remains unverified. On the independent sample, median
runtime was 13.855 s versus 4.302 s for Tesseract.js on the measured Mac, so this
change does not establish a speed advantage.

See the [full results and limitations](docs/kanpo_layout_accuracy_results.md) and
[frozen inputs, raw outputs and reproduction steps](docs/benchmarks/kanpo-layout/README.md).
Use `recognize-page --trace-json path/to/trace.json` to save selection, crop,
recognition and reading-order diagnostics alongside the usual TXT output.

## Setup

```bash
cargo build
```

With ONNX:

```bash
cargo build --features onnx
```

## Main commands

Tests:

```bash
cargo test
```

Benchmarks:

```bash
cargo bench --bench line_segment_bench -- --noplot
cargo bench --bench parseq_decode_bench -- --noplot
cargo bench --bench parseq_preprocess_bench -- --noplot
```

Examples (real OCR; ONNX feature required):

```bash
cargo run --features onnx -- recognize --image tests/fixtures/scaned0.png

cargo run --features onnx -- recognize-page --image tests/fixtures/scaned0.png \
  --output-txt tmp/scaned0.recognize-page.txt

cargo run --features onnx -- recognize-page --image tests/fixtures/scaned0.png \
  --output-docx tmp/scaned0.recognize-page.docx

cargo run --features onnx -- recognize-page --image tests/fixtures/scaned0.png \
  --output-txt tmp/scaned0.recognize-page.cascade.txt

# Fall back to simple line extraction by disabling DEIM detection
cargo run --features onnx -- recognize-page --image tests/fixtures/scaned0.png \
  --use-deim-detection=false \
  --output-txt tmp/scaned0.no-deim.txt

# Disable half-split re-recognition when a line is too long
cargo run --features onnx -- recognize-page --image tests/fixtures/scaned0.png \
  --split-long-lines=false \
  --output-txt tmp/scaned0.no-split.txt

# Enable candidate re-scoring (quality boost; slower)
cargo run --features onnx -- recognize-page --image tests/fixtures/scaned0.png \
  --quality-boost=true \
  --output-txt tmp/scaned0.quality-boost.txt

# Disable article-structure-based post-processing
cargo run --features onnx -- recognize-page --image tests/fixtures/scaned0.png \
  --structure-rules=false \
  --output-txt tmp/scaned0.no-structure-rules.txt

# Apply domain-specific corrections via rule-pack
cargo run --features onnx -- recognize-page --image tests/fixtures/scaned0.png \
  --rule-pack docs/rule_pack.example.yaml \
  --output-txt tmp/scaned0.rule-pack.txt

# Use the tuned rule-pack for the scaned0 fixture
cargo run --features onnx -- recognize-page --image tests/fixtures/scaned0.png \
  --rule-pack docs/rule_pack.scaned0.yaml \
  --output-txt tmp/scaned0.scaned0-pack.txt

cargo run --features onnx -- recognize-page --image tests/fixtures/scaned0.png \
  --post-dict docs/post_dict.example.yaml \
  --output-txt tmp/scaned0.recognize-page.dict.txt

cargo run --features onnx --example pdf_to_image_real_ocr -- \
  --pdf tests/fixtures/handwritten_stamp.pdf \
  --output-dir tmp/pdf-real-ocr2/out
```

## License

This repository is provided under **Creative Commons Attribution 4.0 International (CC BY 4.0)**.  
See `LICENSE` for details.

## Credits

This implementation is a Rust port PoC that references the design, model operations, and output format of the `ndlocr-lite` project.  
Upstream: [ndlocr-lite](https://github.com/ndl-lab/ndlocr-lite)

When distributing or modifying, comply with CC BY 4.0 and give appropriate credit (upstream project name, reference URL, and whether changes were made).

## Official upstream compatibility

Reviewed official Python source: `ndl-lab/ndlocr-lite@d001dad2013ceabb5736ff8d637474e443eb73c2`
(latest reviewed release: 1.3.1). Models are unchanged from 1.2.1; PARSeq now uses
the official `height > width * 0.8` rotation condition. This does not imply full
GUI/PDF/Markdown feature parity. See [the audit and update procedure](docs/ocr_pipeline_compatibility.md).

Fetch pinned models, charset and license notices with:

```sh
python3 tools/fetch_upstream_assets.py models
python3 tools/fetch_upstream_assets.py models --offline
```

Reusable horizontal-line preprocessing is available through
`pipeline::line_chunks::split_line`; both simple line detectors preserve nearby
thin edge glyphs. Domain-specific text replacement remains external rule-pack policy.
