#!/usr/bin/env bash
set -euo pipefail

# Build Rust OCR predictions for evaluation.
#
# Example:
#   tools/run_eval_rust_predictions.sh \
#     --input-dir tests/fixtures/eval/images \
#     --output-dir tmp/eval/rust \
#     --ort-lib-dir third_party/onnxruntime-macos-arm64/lib

INPUT_DIR=""
OUTPUT_DIR=""
MODEL="models/parseq-ndl-24x768-100-tiny-153epoch-tegaki3-r8data-202604.onnx"
CHARSET="ndlocr/src/config/NDLmoji.yaml"
THRESHOLD="220"
RULE_PACK=""
LINE_CROP_PADDING=""
ORT_LIB_DIR=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --input-dir)
      INPUT_DIR="$2"
      shift 2
      ;;
    --output-dir)
      OUTPUT_DIR="$2"
      shift 2
      ;;
    --model)
      MODEL="$2"
      shift 2
      ;;
    --charset)
      CHARSET="$2"
      shift 2
      ;;
    --binarize-threshold)
      THRESHOLD="$2"
      shift 2
      ;;
    --rule-pack)
      RULE_PACK="$2"
      shift 2
      ;;
    --line-crop-padding)
      LINE_CROP_PADDING="$2"
      shift 2
      ;;
    --ort-lib-dir)
      ORT_LIB_DIR="$2"
      shift 2
      ;;
    *)
      echo "unknown option: $1" >&2
      exit 1
      ;;
  esac
done

if [[ -z "$INPUT_DIR" || -z "$OUTPUT_DIR" ]]; then
  echo "usage: $0 --input-dir DIR --output-dir DIR [--model PATH] [--charset PATH] [--binarize-threshold N] [--rule-pack PATH] [--line-crop-padding N] [--ort-lib-dir DIR]" >&2
  exit 1
fi

mkdir -p "$OUTPUT_DIR"

shopt -s nullglob
images=("$INPUT_DIR"/*.png "$INPUT_DIR"/*.jpg "$INPUT_DIR"/*.jpeg "$INPUT_DIR"/*.ppm)
if [[ ${#images[@]} -eq 0 ]]; then
  echo "no images found under: $INPUT_DIR" >&2
  exit 1
fi

for image in "${images[@]}"; do
  stem="$(basename "$image")"
  stem="${stem%.*}"
  out_txt="$OUTPUT_DIR/$stem.txt"
  echo "[rust] $image -> $out_txt"
  cmd=(
    env
    cargo run --features onnx -- recognize-page
    --image "$image"
    --model "$MODEL"
    --charset "$CHARSET"
    --binarize-threshold "$THRESHOLD"
  )
  if [[ -n "$ORT_LIB_DIR" ]]; then
    cmd=(env DYLD_LIBRARY_PATH="$ORT_LIB_DIR" "${cmd[@]:1}")
  fi
  if [[ -n "$RULE_PACK" ]]; then
    cmd+=(--rule-pack "$RULE_PACK")
  fi
  if [[ -n "$LINE_CROP_PADDING" ]]; then
    cmd+=(--line-crop-padding "$LINE_CROP_PADDING")
  fi
  cmd+=(--output-txt "$out_txt")
  "${cmd[@]}" >/dev/null
done

echo "done: wrote ${#images[@]} files to $OUTPUT_DIR"
