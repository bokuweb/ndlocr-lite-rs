#!/usr/bin/env bash
set -euo pipefail
root=$(cd "$(dirname "$0")/.." && pwd)
out="${1:-$root/web/dist}"
mkdir -p "$out"
out=$(cd "$out" && pwd)
cd "$root"
target_dir="${CARGO_TARGET_DIR:-$root/target}"
bindgen=wasm-bindgen
version=0.2.121
if ! command -v wasm-bindgen >/dev/null || [ "$(wasm-bindgen --version)" != "wasm-bindgen $version" ]; then
  case "$(uname -s)-$(uname -m)" in
    Linux-x86_64) triple=x86_64-unknown-linux-musl ;;
    Linux-aarch64) triple=aarch64-unknown-linux-gnu ;;
    Darwin-arm64) triple=aarch64-apple-darwin ;;
    Darwin-x86_64) triple=x86_64-apple-darwin ;;
    *) echo 'Install wasm-bindgen-cli 0.2.121 for your platform' >&2; exit 1 ;;
  esac
  directory="$target_dir/wasm-bindgen-$version"
  if [ ! -x "$directory/wasm-bindgen" ]; then
    mkdir -p "$directory"
    curl -fsSL "https://github.com/wasm-bindgen/wasm-bindgen/releases/download/$version/wasm-bindgen-$version-$triple.tar.gz" | tar xz -C "$directory" --strip-components=1
  fi
  bindgen="$directory/wasm-bindgen"
fi
cargo build --release --target wasm32-unknown-unknown --features web --lib
mkdir -p web/generated
"$bindgen" --target web --out-dir web/generated --out-name ndlocr_wasm "$target_dir/wasm32-unknown-unknown/release/ndlocr_lite_rs.wasm"
node web/build.mjs "$out"
