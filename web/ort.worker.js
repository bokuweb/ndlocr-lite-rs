import * as ort from 'onnxruntime-web/wasm';
import { modelUrl, pack, writeResponse } from './protocol.js';

let base;
const sessions = new Map();
const texts = new Map();
const retained = new Map();
function progress(message) { self.postMessage({ progress: message }); }
async function session(path) {
  const url = modelUrl(base, path);
  if (!sessions.has(url)) {
    progress(`Loading ${path.split('/').pop()}`);
    const promise = ort.InferenceSession.create(url, { executionProviders: ['wasm'], graphOptimizationLevel: 'all' });
    sessions.set(url, promise);
    promise.catch(() => sessions.delete(url));
  }
  return sessions.get(url);
}
function metadata(values) {
  return values.map(value => ({ name: value.name, shape: value.isTensor ? value.shape.map(d => typeof d === 'number' ? d : -1) : null }));
}
async function execute(request, data) {
  switch (request.op) {
    // Known model existence is resolved at load, preserving the load error rather than silently suppressing OCR.
    case 'is_file': return pack({ exists: true });
    case 'read': {
      const url = modelUrl(base, request.path);
      if (!texts.has(url)) {
        const response = await fetch(url);
        if (!response.ok) throw new Error(`Cannot fetch ${url}: HTTP ${response.status}`);
        texts.set(url, await response.text());
      }
      return pack({ text: texts.get(url) });
    }
    case 'load': {
      const model = await session(request.path);
      return pack({ inputs: metadata(model.inputMetadata), outputs: metadata(model.outputMetadata) });
    }
    case 'run': {
      const model = await session(request.path);
      const feeds = {};
      for (const input of request.inputs) {
        const bytes = data.slice(input.offset, input.offset + input.length).buffer;
        const values = input.type === 'float32' ? new Float32Array(bytes) : new BigInt64Array(bytes);
        feeds[input.name] = new ort.Tensor(input.type, values, input.dims);
      }
      progress(`Recognizing ${request.inputs[0].dims[0]} lines`);
      let outputs = {};
      try {
        outputs = await model.run(feeds);
        return pack({}, Object.entries(outputs).map(([name, tensor]) => ({ name, dims: tensor.dims, type: tensor.type, data: tensor.data })));
      } finally {
        for (const tensor of Object.values(feeds)) tensor.dispose();
        for (const tensor of Object.values(outputs)) tensor.dispose();
      }
    }
    default: throw new Error(`Unknown bridge operation ${request.op}`);
  }
}
async function receive({ data: message }) {
  let response;
  try {
    if (message.drain) {
      response = retained.get(message.id);
      retained.delete(message.id);
      if (!response) throw new Error('No retained inference response');
    } else {
      response = { bytes: await execute(message.request, message.data), failed: false };
    }
  } catch (error) { response = { bytes: pack({ error: String(error) }), failed: true }; }
  if (response.bytes.length > message.sab.byteLength - 16) retained.set(message.id, response);
  writeResponse(message.sab, response.bytes, response.failed);
}
self.onmessage = ({ data }) => {
  try {
    base = data.modelBaseUrl;
    ort.env.wasm.numThreads = Math.max(1, Math.min(8, data.numThreads | 0));
    ort.env.wasm.wasmPaths = {
      wasm: new URL('./ort-wasm-simd-threaded.wasm', import.meta.url).href,
      mjs: new URL('./ort-wasm-simd-threaded.mjs', import.meta.url).href,
    };
    data.port.onmessage = receive;
    self.postMessage({ id: data.id, result: true });
  } catch (error) { self.postMessage({ id: data.id, error: String(error) }); }
};
