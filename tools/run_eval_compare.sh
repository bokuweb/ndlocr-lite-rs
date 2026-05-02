#!/usr/bin/env bash
set -euo pipefail

TRUTH_DIR=""
RUST_PRED_DIR=""
ORIGINAL_PRED_DIR=""
OUTPUT_DIR=""
NORMALIZE="basic"
REQUIRE_ALL="false"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --truth-dir)
      TRUTH_DIR="$2"
      shift 2
      ;;
    --rust-pred-dir)
      RUST_PRED_DIR="$2"
      shift 2
      ;;
    --original-pred-dir)
      ORIGINAL_PRED_DIR="$2"
      shift 2
      ;;
    --output-dir)
      OUTPUT_DIR="$2"
      shift 2
      ;;
    --normalize)
      NORMALIZE="$2"
      shift 2
      ;;
    --require-all)
      REQUIRE_ALL="true"
      shift 1
      ;;
    *)
      echo "unknown option: $1" >&2
      exit 1
      ;;
  esac
done

if [[ -z "$TRUTH_DIR" || -z "$RUST_PRED_DIR" || -z "$ORIGINAL_PRED_DIR" || -z "$OUTPUT_DIR" ]]; then
  echo "usage: $0 --truth-dir DIR --rust-pred-dir DIR --original-pred-dir DIR --output-dir DIR [--normalize none|basic|strict] [--require-all]" >&2
  exit 1
fi

mkdir -p "$OUTPUT_DIR"

require_all_arg=()
if [[ "$REQUIRE_ALL" == "true" ]]; then
  require_all_arg+=(--require-all)
fi

python3 tools/eval_ocr_metrics.py \
  --truth-dir "$TRUTH_DIR" \
  --system rust="$RUST_PRED_DIR" \
  --system original="$ORIGINAL_PRED_DIR" \
  --normalize "$NORMALIZE" \
  "${require_all_arg[@]}" \
  --output-csv "$OUTPUT_DIR/report.csv" \
  --output-md "$OUTPUT_DIR/report.md"

echo "done: $OUTPUT_DIR/report.csv and $OUTPUT_DIR/report.md"
