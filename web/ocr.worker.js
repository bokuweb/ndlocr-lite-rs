import init, { recognize_page_gray_u8 } from './generated/ndlocr_wasm.js';
import { CONTROL_BYTES } from './protocol.js';

let port;
let timeoutMs;
let sab = new SharedArrayBuffer(CONTROL_BYTES + 1024 * 1024);
let sequence = 0;
function bridge(request, data) {
  const id = ++sequence;
  let control = new Int32Array(sab, 0, 4);
  Atomics.store(control, 0, 0);
  const copy = data.slice();
  port.postMessage({ id, request: JSON.parse(request), data: copy, sab }, [copy.buffer]);
  for (;;) {
    if (Atomics.wait(control, 0, 0, timeoutMs) === 'timed-out') throw new Error('Inference bridge timed out');
    const status = Atomics.load(control, 0);
    const length = Atomics.load(control, 1);
    if (status === 2) {
      if (length <= 0 || length > 1024 * 1024 * 1024) throw new Error('Inference response exceeds 1 GiB');
      sab = new SharedArrayBuffer(CONTROL_BYTES + length);
      control = new Int32Array(sab, 0, 4);
      port.postMessage({ id, drain: true, sab });
      continue;
    }
    if (status !== 1 && status !== 3) throw new Error('Invalid inference bridge state');
    return new Uint8Array(sab, CONTROL_BYTES, length).slice();
  }
}
self.onmessage = async ({ data }) => {
  try {
    if (data.type === 'init') {
      port = data.port; timeoutMs = data.timeoutMs;
      globalThis.__ndlocrRequest = bridge;
      await init();
      self.postMessage({ id: data.id, result: true });
    } else if (data.type === 'recognize') {
      const result = JSON.parse(recognize_page_gray_u8(data.gray, data.width, data.height));
      self.postMessage({ id: data.id, result });
    }
  } catch (error) { self.postMessage({ id: data.id, error: String(error) }); }
};
