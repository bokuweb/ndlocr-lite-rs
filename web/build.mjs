import { build } from 'esbuild';
import { copyFile, mkdir } from 'node:fs/promises';
import { fileURLToPath } from 'node:url';
import path from 'node:path';
const root = fileURLToPath(new URL('./', import.meta.url));
const out = path.resolve(process.argv[2] || path.join(root, 'dist'));
await mkdir(out, { recursive: true });
await build({ entryPoints: ['ndlocr.js', 'ocr.worker.js', 'ort.worker.js'].map(p => path.join(root, p)), outdir: out, bundle: true, format: 'esm', platform: 'browser', target: 'es2022' });
for (const name of ['ort-wasm-simd-threaded.wasm', 'ort-wasm-simd-threaded.mjs']) {
  await copyFile(path.join(root, 'node_modules/onnxruntime-web/dist', name), path.join(out, name));
}
await copyFile(path.join(root, 'generated/ndlocr_wasm_bg.wasm'), path.join(out, 'ndlocr_wasm_bg.wasm'));
await copyFile(path.join(root, 'third_party/onnxruntime-LICENSE'), path.join(out, 'onnxruntime-LICENSE'));
await copyFile(path.join(root, 'ndlocr.d.ts'), path.join(out, 'ndlocr.d.ts'));
console.log(`Browser OCR package: ${out}`);
