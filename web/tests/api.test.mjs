import { test } from 'node:test';
import assert from 'node:assert/strict';
import { createRecognizer } from '../ndlocr.js';

test('isolation failure is actionable', async () => {
  globalThis.crossOriginIsolated = false;
  await assert.rejects(createRecognizer({ modelBaseUrl: '/models' }), /COOP\/COEP/);
});
test('requests queue, retain input ownership and stop after dispose', async () => {
  globalThis.crossOriginIsolated = true;
  globalThis.location = { href: 'https://example.org/sub/' };
  const workers = [];
  class FakeWorker {
    constructor() { workers.push(this); this.active = 0; }
    terminate() { this.terminated = true; }
    postMessage(message) {
      if (message.type === 'init') { message.port.close(); queueMicrotask(() => this.onmessage({data: {id: message.id, result: true}})); return; }
      assert.equal(++this.active, 1);
      setTimeout(() => { --this.active; this.onmessage({data: {id: message.id, result: {width: message.width}}}); }, 10);
    }
  }
  globalThis.Worker = FakeWorker;
  const recognizer = await createRecognizer({ modelBaseUrl: './models' });
  const gray = new Uint8Array([1,2]);
  const values = await Promise.all([recognizer.recognizePage(gray, 2, 1), recognizer.recognizePage(gray, 1, 2)]);
  assert.deepEqual(values, [{width: 2}, {width: 1}]);
  assert.equal(gray.byteLength, 2);
  await assert.rejects(recognizer.recognizePage(gray, 3, 1), /matching/);
  const running = recognizer.recognizePage(gray, 2, 1);
  recognizer.dispose();
  await assert.rejects(running, /disposed/);
  assert.ok(workers.every(worker => worker.terminated));
});
test('worker crashes reject active and queued requests and terminate both workers', async () => {
  const workers = [];
  globalThis.Worker = class {
    constructor() { workers.push(this); }
    terminate() { this.terminated = true; }
    postMessage(message) {
      if (message.type === 'init') { message.port.close(); queueMicrotask(() => this.onmessage({data:{id:message.id,result:true}})); }
      else queueMicrotask(() => this.onerror({message:'inference crashed'}));
    }
  };
  const recognizer = await createRecognizer({modelBaseUrl:'/models'});
  const outcomes = await Promise.allSettled([recognizer.recognizePage(new Uint8Array(1),1,1), recognizer.recognizePage(new Uint8Array(1),1,1)]);
  assert.ok(outcomes.every(result => result.status === 'rejected'));
  assert.match(outcomes[0].reason.message, /inference crashed/);
  assert.ok(workers.every(worker => worker.terminated));
});
test('initialization has a finite timeout', async () => {
  globalThis.Worker = class { terminate() {} postMessage(message) {message.port?.close();} };
  await assert.rejects(createRecognizer({modelBaseUrl:'/models', timeoutMs:10}), /timed out/);
});
