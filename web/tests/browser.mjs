// Real models, real Chromium, the same package the consumer copies. Run after build.
import { chromium } from '@playwright/test';
import assert from 'node:assert/strict';
import { spawn } from 'node:child_process';
import { once } from 'node:events';
import { writeFile } from 'node:fs/promises';
const root = new URL('../../', import.meta.url);
const port = 18811;
const server = spawn('python3', ['examples/web/serve.py', '--port', String(port)], { cwd: root, stdio: ['ignore', 'pipe', 'pipe'] });
let browser;
try {
  await Promise.race([once(server.stdout, 'data'), once(server, 'exit').then(() => {throw new Error('Sample server exited');})]);
  browser = await chromium.launch({headless: true});
  const page = await browser.newPage();
  const errors = [];
  page.on('pageerror', error => errors.push(String(error)));
  await page.goto(`http://127.0.0.1:${port}/examples/web/`);
  assert.equal(await page.evaluate(() => crossOriginIsolated), true);
  const runs = await page.evaluate(async () => {
    const results = [];
    for (let i = 0; i < 2; i++) results.push(await window.runOcr('../../tests/fixtures/kanpo_layout/20261007g002210006.png'));
    return results;
  });
  for (const { trace } of runs) {
    assert.equal(trace.schema_version, 1);
    assert.equal(trace.width, 3509); assert.equal(trace.height, 2480);
    assert.ok(trace.stages.reading_order.length > 150);
    assert.ok(trace.body.includes('破産手続を開始する'));
    assert.ok(trace.body.includes('上野優磨'));
  }
  assert.equal(runs[0].trace.body, runs[1].trace.body);
  const progress = await page.locator('#progress').textContent();
  for (const model of ['deim-s-', '24x256-', '24x384-', '24x768-']) assert.ok(progress.includes(model));
  assert.equal((progress.match(/Loading /g) || []).length, 4, 'second page must reuse model sessions');
  await assert.rejects(page.evaluate(async () => {
    const {createRecognizer} = await import('../../web/dist/ndlocr.js');
    const recognizer = await createRecognizer({modelBaseUrl:'/missing-models/', timeoutMs:10000});
    try {await recognizer.recognizePage(new Uint8Array(64).fill(255),8,8);}
    finally {recognizer.dispose();}
  }), /HTTP|fetch|failed/i);
  assert.deepEqual(errors, []);
  const summary = { browser: browser.version(), runs: runs.map(({seconds,totalSeconds,trace}) => ({seconds,totalSeconds,characters:trace.body.length,lines:trace.stages.reading_order.length})) };
  console.log(JSON.stringify(summary));
  if (process.env.NDLOCR_BROWSER_EVIDENCE) await writeFile(process.env.NDLOCR_BROWSER_EVIDENCE, JSON.stringify({summary,runs},null,2));
} finally {
  await browser?.close();
  server.kill();
}
