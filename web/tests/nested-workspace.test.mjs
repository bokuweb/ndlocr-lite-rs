import { test } from 'node:test';
import assert from 'node:assert/strict';
import { mkdtempSync, realpathSync, mkdirSync, readFileSync, writeFileSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { spawnSync } from 'node:child_process';

test('the source package builds independently inside a host Cargo workspace', () => {
  const root = mkdtempSync(join(tmpdir(), 'ndlocr-host-workspace-'));
  try {
    writeFileSync(join(root, 'Cargo.toml'), '[workspace]\nmembers = []\n');
    const nested = join(root, 'target', 'ndlocr-source');
    mkdirSync(join(nested, 'src'), {recursive:true});
    const manifest = readFileSync(new URL('../../Cargo.toml', import.meta.url), 'utf8');
    writeFileSync(join(nested, 'Cargo.toml'), manifest);
    writeFileSync(join(nested, 'src', 'lib.rs'), 'pub fn fixture() {}');
    for (const [section, directory] of [['example','examples'], ['bench','benches']]) {
      mkdirSync(join(nested, directory));
      const pattern = new RegExp(`\\[\\[${section}\\]\\]\\s*name = "([^"]+)"`, 'g');
      for (const [,name] of manifest.matchAll(pattern)) writeFileSync(join(nested, directory, `${name}.rs`), 'fn main() {}');
    }
    const result = spawnSync('cargo', ['metadata', '--no-deps', '--offline', '--format-version', '1'], {cwd:nested,encoding:'utf8',timeout:30000});
    assert.equal(result.status, 0, result.stderr);
    const metadata = JSON.parse(result.stdout);
    assert.equal(metadata.workspace_root, realpathSync(nested));
  } finally {rmSync(root,{recursive:true,force:true});}
});
