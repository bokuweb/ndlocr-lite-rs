#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
OUT_DIR="${ROOT_DIR}/models"

usage() {
  cat <<'EOF'
Usage:
  tools/build_local_models.sh \
    --deim-repo /path/to/DEIMv2 \
    --deim-config /path/to/deimv2_dinov3_s_coco_r4_800.yml \
    --deim-ckpt /path/to/deim_last.pth \
    --parseq-ckpt30 /path/to/parseq_30.ckpt \
    --parseq-ckpt50 /path/to/parseq_50.ckpt \
    --parseq-ckpt100 /path/to/parseq_100.ckpt \
    [--charset-yaml /path/to/NDLmoji.yaml] \
    [--out-dir /path/to/models]

This script converts local checkpoints to ONNX files:
  - deim-s-1024x1024.onnx
  - parseq-ndl-24x256-30-tiny-189epoch-tegaki3-r8data-202604.onnx
  - parseq-ndl-24x384-50-tiny-300epoch-tegaki3-r8data-202604.onnx
  - parseq-ndl-24x768-100-tiny-153epoch-tegaki3-r8data-202604.onnx

Requirements:
  - Python environment with torch/strhub/yaml and DEIMv2 export dependencies
  - ndlocr training helper files already copied into DEIMv2/parseq repos per ndlocr/train/README.md
EOF
}

DEIM_REPO=""
DEIM_CONFIG=""
DEIM_CKPT=""
PARSEQ_CKPT30=""
PARSEQ_CKPT50=""
PARSEQ_CKPT100=""
CHARSET_YAML="${ROOT_DIR}/ndlocr/src/config/NDLmoji.yaml"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --deim-repo) DEIM_REPO="$2"; shift 2 ;;
    --deim-config) DEIM_CONFIG="$2"; shift 2 ;;
    --deim-ckpt) DEIM_CKPT="$2"; shift 2 ;;
    --parseq-ckpt30) PARSEQ_CKPT30="$2"; shift 2 ;;
    --parseq-ckpt50) PARSEQ_CKPT50="$2"; shift 2 ;;
    --parseq-ckpt100) PARSEQ_CKPT100="$2"; shift 2 ;;
    --charset-yaml) CHARSET_YAML="$2"; shift 2 ;;
    --out-dir) OUT_DIR="$2"; shift 2 ;;
    -h|--help) usage; exit 0 ;;
    *) echo "Unknown arg: $1" >&2; usage; exit 1 ;;
  esac
done

[[ -n "${DEIM_REPO}" ]] || { echo "--deim-repo is required" >&2; exit 1; }
[[ -n "${DEIM_CONFIG}" ]] || { echo "--deim-config is required" >&2; exit 1; }
[[ -n "${DEIM_CKPT}" ]] || { echo "--deim-ckpt is required" >&2; exit 1; }
[[ -n "${PARSEQ_CKPT30}" ]] || { echo "--parseq-ckpt30 is required" >&2; exit 1; }
[[ -n "${PARSEQ_CKPT50}" ]] || { echo "--parseq-ckpt50 is required" >&2; exit 1; }
[[ -n "${PARSEQ_CKPT100}" ]] || { echo "--parseq-ckpt100 is required" >&2; exit 1; }

[[ -f "${DEIM_CONFIG}" ]] || { echo "not found: ${DEIM_CONFIG}" >&2; exit 1; }
[[ -f "${DEIM_CKPT}" ]] || { echo "not found: ${DEIM_CKPT}" >&2; exit 1; }
[[ -f "${PARSEQ_CKPT30}" ]] || { echo "not found: ${PARSEQ_CKPT30}" >&2; exit 1; }
[[ -f "${PARSEQ_CKPT50}" ]] || { echo "not found: ${PARSEQ_CKPT50}" >&2; exit 1; }
[[ -f "${PARSEQ_CKPT100}" ]] || { echo "not found: ${PARSEQ_CKPT100}" >&2; exit 1; }
[[ -f "${CHARSET_YAML}" ]] || { echo "not found: ${CHARSET_YAML}" >&2; exit 1; }

mkdir -p "${OUT_DIR}"

echo "[1/4] Export DEIM ONNX..."
python3 "${ROOT_DIR}/ndlocr/train/deimv2code/part2/tools/deployment/export_onnx.py" \
  -c "${DEIM_CONFIG}" \
  -r "${DEIM_CKPT}" \
  --check
cp -f "${DEIM_CKPT%.pth}.onnx" "${OUT_DIR}/deim-s-1024x1024.onnx"

echo "[2/4] Export PARSeq-30 ONNX..."
python3 "${ROOT_DIR}/tools/export_parseq_onnx.py" \
  --checkpoint "${PARSEQ_CKPT30}" \
  --charset-yaml "${CHARSET_YAML}" \
  --output "${OUT_DIR}/parseq-ndl-24x256-30-tiny-189epoch-tegaki3-r8data-202604.onnx" \
  --height 24 --width 256

echo "[3/4] Export PARSeq-50 ONNX..."
python3 "${ROOT_DIR}/tools/export_parseq_onnx.py" \
  --checkpoint "${PARSEQ_CKPT50}" \
  --charset-yaml "${CHARSET_YAML}" \
  --output "${OUT_DIR}/parseq-ndl-24x384-50-tiny-300epoch-tegaki3-r8data-202604.onnx" \
  --height 24 --width 384

echo "[4/4] Export PARSeq-100 ONNX..."
python3 "${ROOT_DIR}/tools/export_parseq_onnx.py" \
  --checkpoint "${PARSEQ_CKPT100}" \
  --charset-yaml "${CHARSET_YAML}" \
  --output "${OUT_DIR}/parseq-ndl-24x768-100-tiny-153epoch-tegaki3-r8data-202604.onnx" \
  --height 24 --width 768

echo "Done. ONNX files are in: ${OUT_DIR}"
