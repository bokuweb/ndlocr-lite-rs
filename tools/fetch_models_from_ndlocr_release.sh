#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
WORK_DIR="${ROOT_DIR}/tmp/ndlocr-release-fetch"
OUT_DIR="${ROOT_DIR}/models"
REPO="ndl-lab/ndlocr-lite"
TAG="1.2.1"
ASSET_GLOB="*windows.zip"
ZIP_FILE=""

usage() {
  cat <<'EOF'
Fetch bundled ONNX models from ndlocr-lite release package.

Usage:
  tools/fetch_models_from_ndlocr_release.sh [--tag 1.2.1] [--out-dir models]
  tools/fetch_models_from_ndlocr_release.sh --zip-file /path/to/ndlocr_lite_v1.2.1_windows.zip

Requirements:
  - gh CLI
  - unzip
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --tag) TAG="$2"; shift 2 ;;
    --out-dir) OUT_DIR="$2"; shift 2 ;;
    --zip-file) ZIP_FILE="$2"; shift 2 ;;
    -h|--help) usage; exit 0 ;;
    *) echo "Unknown arg: $1" >&2; usage; exit 1 ;;
  esac
done

mkdir -p "${WORK_DIR}" "${OUT_DIR}"
rm -f "${WORK_DIR}/"*

if [[ -z "${ZIP_FILE}" ]]; then
  echo "[1/4] Download release asset (${TAG})..."
  gh release download "${TAG}" -R "${REPO}" -p "${ASSET_GLOB}" -D "${WORK_DIR}"
  ZIP_FILE="$(ls "${WORK_DIR}"/*windows.zip | head -n 1)"
else
  [[ -f "${ZIP_FILE}" ]] || { echo "zip not found: ${ZIP_FILE}" >&2; exit 1; }
  echo "[1/4] Use local zip: ${ZIP_FILE}"
fi
APP_ZIP="${WORK_DIR}/app.zip"

echo "[2/4] Extract nested app.zip..."
# Some zip variants return non-zero with separator warnings; verify by file existence.
unzip -o -j "${ZIP_FILE}" "*app.zip" -d "${WORK_DIR}" >/dev/null || true
[[ -f "${APP_ZIP}" ]] || { echo "failed to extract nested app.zip from ${ZIP_FILE}" >&2; exit 1; }

echo "[3/4] Extract ONNX models..."
unzip -o -j "${APP_ZIP}" \
  "src/model/deim-s-1024x1024.onnx" \
  "src/model/parseq-ndl-24x256-30-tiny-189epoch-tegaki3-r8data-202604.onnx" \
  "src/model/parseq-ndl-24x384-50-tiny-300epoch-tegaki3-r8data-202604.onnx" \
  "src/model/parseq-ndl-24x768-100-tiny-153epoch-tegaki3-r8data-202604.onnx" \
  -d "${OUT_DIR}" >/dev/null

echo "[4/4] Done. Extracted files:"
ls -lh "${OUT_DIR}"/deim-s-1024x1024.onnx \
       "${OUT_DIR}"/parseq-ndl-24x256-30-tiny-189epoch-tegaki3-r8data-202604.onnx \
       "${OUT_DIR}"/parseq-ndl-24x384-50-tiny-300epoch-tegaki3-r8data-202604.onnx \
       "${OUT_DIR}"/parseq-ndl-24x768-100-tiny-153epoch-tegaki3-r8data-202604.onnx
