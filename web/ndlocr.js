/** Browser-only page OCR. Host must provide cross-origin isolation. */
export async function createRecognizer({ modelBaseUrl, onProgress, numThreads = 2, timeoutMs = 300000 } = {}) {
  if (!globalThis.crossOriginIsolated || typeof SharedArrayBuffer === 'undefined') {
    throw new Error('Browser OCR requires crossOriginIsolated (COOP/COEP) and SharedArrayBuffer');
  }
  const base = new URL(modelBaseUrl, globalThis.location.href).href;
  const ort = new Worker(new URL('./ort.worker.js', import.meta.url), { type: 'module' });
  let ocr;
  try { ocr = new Worker(new URL('./ocr.worker.js', import.meta.url), { type: 'module' }); }
  catch (error) { ort.terminate(); throw error; }
  const channel = new MessageChannel();
  const pending = new Map();
  let disposed = false;
  let counter = 0;
  let queue = Promise.resolve();
  function shutdown(error) {
    if (disposed) return;
    disposed = true;
    ort.terminate(); ocr.terminate();
    for (const { reject, timer } of pending.values()) { clearTimeout(timer); reject(error); }
    pending.clear();
  }
  function request(worker, message, transfer = []) {
    if (disposed) return Promise.reject(new Error('Recognizer is disposed'));
    const id = ++counter;
    return new Promise((resolve, reject) => {
      const timer = setTimeout(() => shutdown(new Error('OCR worker timed out')), timeoutMs);
      pending.set(id, { resolve, reject, timer });
      try { worker.postMessage({ ...message, id }, transfer); }
      catch (error) { clearTimeout(timer); pending.delete(id); reject(error); }
    });
  }
  function receive({ data }) {
    if (data.progress) { onProgress?.(data.progress); return; }
    const entry = pending.get(data.id);
    if (!entry) return;
    clearTimeout(entry.timer); pending.delete(data.id);
    if (data.error) entry.reject(new Error(data.error)); else entry.resolve(data.result);
  }
  for (const worker of [ocr, ort]) {
    worker.onmessage = receive;
    worker.onerror = event => shutdown(new Error(event.message || 'OCR worker failed'));
    worker.onmessageerror = () => shutdown(new Error('OCR worker message could not be decoded'));
  }
  try {
    await request(ort, { type: 'init', port: channel.port1, modelBaseUrl: base, numThreads }, [channel.port1]);
    await request(ocr, { type: 'init', port: channel.port2, timeoutMs }, [channel.port2]);
  } catch (error) { shutdown(error); throw error; }
  return {
    recognizePage(gray, width, height) {
      if (!(gray instanceof Uint8Array) || !Number.isSafeInteger(width) || !Number.isSafeInteger(height) || width <= 0 || height <= 0 || width > 0xffffffff || height > 0xffffffff || gray.length !== width * height) {
        return Promise.reject(new Error('Expected grayscale Uint8Array with positive dimensions matching its length'));
      }
      const copy = gray.slice();
      const result = queue.then(() => request(ocr, { type: 'recognize', gray: copy, width, height }, [copy.buffer]));
      queue = result.catch(() => {});
      return result;
    },
    dispose() { shutdown(new Error('Recognizer is disposed')); },
  };
}
