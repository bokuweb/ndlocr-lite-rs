import { test } from 'node:test';
import assert from 'node:assert/strict';
import { pack, unpack, writeResponse, modelUrl } from '../protocol.js';

test('binary tensors preserve f32 and full-width i64 without alignment assumptions', () => {
  const tensors = [
    { name: 'image', dims: [2], type: 'float32', data: new Float32Array([0.125, -23.5]) },
    { name: 'sizes', dims: [2], type: 'int64', data: new BigInt64Array([9223372036854775807n, -8n]) },
  ];
  const decoded = unpack(pack({ ok: true }, tensors));
  assert.equal(decoded.meta.ok, true);
  assert.deepEqual(decoded.tensors.map(t => Array.from(t.data)), tensors.map(t => Array.from(t.data)));
});
test('oversized response signals resize before bytes are copied', () => {
  const sab = new SharedArrayBuffer(24);
  const bytes = new Uint8Array(100).fill(9);
  writeResponse(sab, bytes);
  assert.equal(Atomics.load(new Int32Array(sab, 0, 4), 0), 2);
  assert.equal(Atomics.load(new Int32Array(sab, 0, 4), 1), 100);
  const larger = new SharedArrayBuffer(116);
  writeResponse(larger, bytes);
  assert.equal(Atomics.load(new Int32Array(larger, 0, 4), 0), 1);
  assert.deepEqual(new Uint8Array(larger, 16), bytes);
});
test('models use basename and preserve a host subpath without trailing slash', () => {
  assert.equal(modelUrl('https://example.org/sub/models', 'models/deim.onnx'), 'https://example.org/sub/models/deim.onnx');
  assert.equal(modelUrl('https://example.org/sub/models/', 'ndlocr/src/config/NDLmoji.yaml'), 'https://example.org/sub/models/NDLmoji.yaml');
});
test('worker errors have a distinct wakeup state', () => {
  const sab = new SharedArrayBuffer(100);
  writeResponse(sab, pack({ error: 'failed' }), true);
  assert.equal(Atomics.load(new Int32Array(sab, 0, 4), 0), 3);
});
