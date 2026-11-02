# Browser OCR package

`web` enables the same synchronous page pipeline with a JS inference backend.
A Rust/WASM OCR worker blocks on SharedArrayBuffer/Atomics; a separate ORT worker
runs onnxruntime-web asynchronously. No service worker is installed by the library.
The host must provide COOP/COEP and `crossOriginIsolated`. Missing isolation is a
clear initialization error. A standalone sample server sends these headers.

Public ESM API:

```js
const recognizer = await createRecognizer({ modelBaseUrl, onProgress });
const trace = await recognizer.recognizePage(gray, width, height);
recognizer.dispose();
```

Input is Uint8Array grayscale, exactly width*height bytes, with positive dimensions.
Requests serialize per recognizer. The source array stays usable by its caller.
Dispose terminates both workers and rejects pending requests; initialization and
worker errors reject promises instead of leaving the UI busy. Bridge waits have a
finite timeout. Progress callbacks receive strings.

Models resolve by basename below modelBaseUrl; charset text and sessions cache per
URL in the ORT worker. Model load failures propagate and are not treated as an
absent detector. The trace includes schema_version/width/height/stages/body exactly
as the native page API does. WASM preprocessing and pool execution are sequential, with batches capped at four
lines to bound logits/bridge memory;
native scheduling remains unchanged.

Wire protocol: each request carries JSON metadata and binary f32/i64 input bytes.
Responses begin with a little-endian u32 JSON-header length, then UTF-8 metadata
and binary tensor bytes. Tensor offsets are relative to the binary payload. SAB
control words are status, required bytes, reserved, reserved. Status 1=complete,
2=resize, 3=error. If a response exceeds capacity, the ORT worker retains that
response; a drain request with a larger SAB copies it without rerunning inference.
The OCR worker copies the response before reusing its buffer. i64 values retain
all 64 bits. Tests cover oversized responses, errors, typed tensor round trips,
URL resolution, isolation, queueing, disposal and browser execution.

Build with `npm ci --prefix web && bash tools/build_web.sh [output-directory]`.
The package bundles pinned onnxruntime-web 1.24.3 and its matching WASM/MJS files,
with no runtime CDN dependency. All worker/WASM URLs resolve relative to modules
using new URL(..., import.meta.url), including under a Vite relative base/subpath.
The output directory can be copied to lawrenceanum/figma/src/wasm/ndlocr/.

Required models under modelBaseUrl:
- deim-s-1024x1024.onnx
- parseq-ndl-24x256-30-tiny-189epoch-tegaki3-r8data-202604.onnx
- parseq-ndl-24x384-50-tiny-300epoch-tegaki3-r8data-202604.onnx
- parseq-ndl-24x768-100-tiny-153epoch-tegaki3-r8data-202604.onnx
- NDLmoji.yaml

Cross-origin model hosting must allow CORS for the application origin. The native
CLI stays on ORT rc.13; browser floating-point results may differ across engines.
A browser page measurement is recorded separately from native byte parity.

Run `python3 examples/web/serve.py` and open `http://127.0.0.1:8811/examples/web/`.

## Verified integration (2026-11-02)

`docs/web-validation.json` records the input/body SHA-256, dimensions, runtime,
measurements and limits. Chromium 147 on Apple M6/32 GiB, ORT Web 1.24.3 with two
threads returned 185 lines / 3,027 characters for a 3509×2480 kanpo page. Repeated
pages took 24.49 and 21.28 seconds in the final smoke test; all four models loaded
only once. Its body matched the saved native CLI output byte for byte. Native
regressions passed and the final CLI matrix matched all 165 baseline files.

The Vite production build was also served at `/sub/`; an image-only kanpo PDF
produced a downloadable DOCX in 35 seconds with OCR text in `word/document.xml`.
These are localhost measurements with other checks running, not R2 transfer
benchmarks or a guarantee for other devices/pages.

Reproduce the browser check (including missing models and session reuse):

```sh
npm ci --prefix web
bash tools/build_web.sh
python3 tools/fetch_upstream_assets.py models
(cd web && npx playwright install chromium)
npm run test:browser --prefix web
```

Generated files: `ndlocr.js`, `ndlocr.d.ts`, `ocr.worker.js`, `ort.worker.js`,
`ndlocr_wasm_bg.wasm`, `ort-wasm-simd-threaded.mjs`,
`ort-wasm-simd-threaded.wasm`, `onnxruntime-LICENSE`. Both ORT runtime assets are
bundled from the same pinned npm package as its JS. Models are separate.
