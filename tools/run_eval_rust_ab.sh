#!/usr/bin/env bash
set -euo pipefail

# Build Rust OCR predictions (baseline + rule-pack) and evaluate CER/WER.
#
# Example:
#   ORT_STRATEGY=system ORT_LIB_LOCATION=third_party/onnxruntime-macos-arm64 \
#   tools/run_eval_rust_ab.sh \
#     --input-dir tests/fixtures/eval/images \
#     --truth-dir tests/fixtures/eval/truth \
#     --output-dir tmp/eval \
#     --rule-pack docs/rule_pack.scaned0.yaml \
#     --ort-lib-dir third_party/onnxruntime-macos-arm64/lib
#     # optional: --line-crop-padding 1

INPUT_DIR=""
TRUTH_DIR=""
OUTPUT_DIR=""
RULE_PACK=""
NORMALIZE="basic"
ORT_LIB_DIR=""
LINE_CROP_PADDING=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --input-dir)
      INPUT_DIR="$2"
      shift 2
      ;;
    --truth-dir)
      TRUTH_DIR="$2"
      shift 2
      ;;
    --output-dir)
      OUTPUT_DIR="$2"
      shift 2
      ;;
    --rule-pack)
      RULE_PACK="$2"
      shift 2
      ;;
    --normalize)
      NORMALIZE="$2"
      shift 2
      ;;
    --ort-lib-dir)
      ORT_LIB_DIR="$2"
      shift 2
      ;;
    --line-crop-padding)
      LINE_CROP_PADDING="$2"
      shift 2
      ;;
    *)
      echo "unknown option: $1" >&2
      exit 1
      ;;
  esac
done

if [[ -z "$INPUT_DIR" || -z "$TRUTH_DIR" || -z "$OUTPUT_DIR" || -z "$RULE_PACK" ]]; then
  echo "usage: $0 --input-dir DIR --truth-dir DIR --output-dir DIR --rule-pack PATH [--normalize none|basic|strict|both] [--ort-lib-dir DIR] [--line-crop-padding N]" >&2
  exit 1
fi

if [[ ! -d "$INPUT_DIR" ]]; then
  echo "input dir not found: $INPUT_DIR" >&2
  exit 1
fi
if [[ ! -d "$TRUTH_DIR" ]]; then
  echo "truth dir not found: $TRUTH_DIR" >&2
  exit 1
fi
if [[ ! -f "$RULE_PACK" ]]; then
  echo "rule-pack not found: $RULE_PACK" >&2
  exit 1
fi
if [[ "$NORMALIZE" != "none" && "$NORMALIZE" != "basic" && "$NORMALIZE" != "strict" && "$NORMALIZE" != "both" ]]; then
  echo "invalid --normalize: $NORMALIZE (expected none|basic|strict|both)" >&2
  exit 1
fi

mkdir -p "$OUTPUT_DIR"
RUST_DIR="$OUTPUT_DIR/rust"
RUST_RULE_PACK_DIR="$OUTPUT_DIR/rust_rule_pack"

ort_args=()
if [[ -n "$ORT_LIB_DIR" ]]; then
  ort_args+=(--ort-lib-dir "$ORT_LIB_DIR")
fi
if [[ -n "$LINE_CROP_PADDING" ]]; then
  ort_args+=(--line-crop-padding "$LINE_CROP_PADDING")
fi

tools/run_eval_rust_predictions.sh \
  --input-dir "$INPUT_DIR" \
  --output-dir "$RUST_DIR" \
  "${ort_args[@]}"

tools/run_eval_rust_predictions.sh \
  --input-dir "$INPUT_DIR" \
  --output-dir "$RUST_RULE_PACK_DIR" \
  --rule-pack "$RULE_PACK" \
  "${ort_args[@]}"

eval_normalizes=("$NORMALIZE")
if [[ "$NORMALIZE" == "both" ]]; then
  eval_normalizes=("basic" "strict")
fi

for nz in "${eval_normalizes[@]}"; do
  python3 tools/eval_ocr_metrics.py \
    --truth-dir "$TRUTH_DIR" \
    --system rust="$RUST_DIR" \
    --system rust_rule_pack="$RUST_RULE_PACK_DIR" \
    --baseline-system rust \
    --normalize "$nz" \
    --require-all \
    --output-csv "$OUTPUT_DIR/report_rust_ab_${nz}.csv" \
    --output-md "$OUTPUT_DIR/report_rust_ab_${nz}.md"
done

if [[ "$NORMALIZE" == "both" ]]; then
  echo "done: $OUTPUT_DIR/report_rust_ab_basic.{csv,md} and $OUTPUT_DIR/report_rust_ab_strict.{csv,md}"
else
  echo "done: $OUTPUT_DIR/report_rust_ab_${NORMALIZE}.csv and $OUTPUT_DIR/report_rust_ab_${NORMALIZE}.md"
fi
