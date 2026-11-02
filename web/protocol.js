const encoder = new TextEncoder();
const decoder = new TextDecoder();
export const CONTROL_BYTES = 16;

export function modelUrl(base, path) {
  const name = path.replaceAll('\\', '/').split('/').pop();
  return new URL(encodeURIComponent(name), base.replace(/\/?$/, '/')).href;
}

export function pack(meta, tensors = []) {
  let offset = 0;
  const descriptors = tensors.map(({ name, dims, type, data }) => {
    if (!['float32', 'int64'].includes(type)) throw new Error(`Unsupported tensor type: ${type}`);
    const info = { name, dims, type, offset, length: data.byteLength };
    offset += data.byteLength;
    return info;
  });
  const header = encoder.encode(JSON.stringify({ ...meta, tensors: descriptors }));
  const result = new Uint8Array(4 + header.length + offset);
  new DataView(result.buffer).setUint32(0, header.length, true);
  result.set(header, 4);
  tensors.forEach((tensor, i) => result.set(new Uint8Array(tensor.data.buffer, tensor.data.byteOffset, tensor.data.byteLength), 4 + header.length + descriptors[i].offset));
  return result;
}

export function unpack(bytes) {
  const length = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength).getUint32(0, true);
  if (length > bytes.length - 4) throw new Error('Truncated bridge header');
  const meta = JSON.parse(decoder.decode(bytes.subarray(4, 4 + length)));
  const tensors = (meta.tensors ?? []).map(({ name, dims, type, offset, length: size }) => {
    const start = 4 + length + offset;
    if (start + size > bytes.length || offset < 0) throw new Error('Truncated bridge tensor');
    const copy = bytes.slice(start, start + size).buffer;
    const data = type === 'float32' ? new Float32Array(copy) : type === 'int64' ? new BigInt64Array(copy) : null;
    if (!data) throw new Error(`Unsupported tensor type: ${type}`);
    if (dims.reduce((n, d) => n * d, 1) !== data.length) throw new Error('Tensor shape mismatch');
    return { name, dims, type, data };
  });
  return { meta, tensors };
}

export function writeResponse(sab, bytes, failed = false) {
  const control = new Int32Array(sab, 0, 4);
  Atomics.store(control, 1, bytes.length);
  if (bytes.length > sab.byteLength - CONTROL_BYTES) {
    Atomics.store(control, 0, 2);
  } else {
    new Uint8Array(sab, CONTROL_BYTES, bytes.length).set(bytes);
    Atomics.store(control, 0, failed ? 3 : 1);
  }
  Atomics.notify(control, 0);
}
