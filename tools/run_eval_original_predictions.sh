#!/usr/bin/env bash
set -euo pipefail

INPUT_DIR=""
OUTPUT_DIR=""
PYTHON_BIN="${PYTHON_BIN:-python3}"
NDLOCR_SRC_DIR=""

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
    --python)
      PYTHON_BIN="$2"
      shift 2
      ;;
    --ndlocr-src)
      NDLOCR_SRC_DIR="$2"
      shift 2
      ;;
    *)
      echo "unknown option: $1" >&2
      exit 1
      ;;
  esac
done

if [[ -z "$INPUT_DIR" || -z "$OUTPUT_DIR" ]]; then
  echo "usage: $0 --input-dir DIR --output-dir DIR [--python PYTHON] [--ndlocr-src DIR]" >&2
  exit 1
fi

if [[ ! -d "$INPUT_DIR" ]]; then
  echo "input dir not found: $INPUT_DIR" >&2
  exit 1
fi

# ocr.py runs with cwd=ndlocr/src; relative paths would break.
INPUT_DIR="$(cd "$INPUT_DIR" && pwd)"
mkdir -p "$OUTPUT_DIR"
OUTPUT_DIR="$(cd "$OUTPUT_DIR" && pwd)"

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
if [[ -z "$NDLOCR_SRC_DIR" ]]; then
  NDLOCR_SRC_DIR="$ROOT_DIR/ndlocr/src"
else
  NDLOCR_SRC_DIR="$(cd "$NDLOCR_SRC_DIR" && pwd)"
fi

if [[ ! -f "$NDLOCR_SRC_DIR/ocr.py" ]]; then
  echo "original script not found: $NDLOCR_SRC_DIR/ocr.py" >&2
  exit 1
fi

(
  cd "$NDLOCR_SRC_DIR"
  "$PYTHON_BIN" ocr.py \
    --sourcedir "$INPUT_DIR" \
    --output "$OUTPUT_DIR"
)

echo "done: original predictions under $OUTPUT_DIR"
