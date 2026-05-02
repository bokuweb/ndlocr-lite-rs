# ndlocr-lite-rs

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
