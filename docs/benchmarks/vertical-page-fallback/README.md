# Vertical fallback: frozen FVA / FVB evidence

The production runtime remains O (`1f240be`, inference equivalent to `73e9ab4`).
FVB is an isolated proposal pending the full eight-route matrix and integration gates.

FVA is held: its 240-input, six-Page-route comparison produced regressions in 10 inputs /
19 input-route pairs. Geometry retained the glyphs, but confident outputs included `AAAA`,
`NNNN`, repeated English words on drawn nonletter controls, and one worse body result.
The official and independent calculations agreed on all 8,640 CSV rows.

FVB adds nonwhitespace scalar coverage of at least half the image's square-anchor count,
plus four distinct Unicode alphanumeric scalars. The original text and confidence are
reused; a rejected attempt returns to the original horizontal fallback. H/O and document
replacement dictionaries are unchanged. Its Mac/debug preflight on 348 inputs × six
Page routes had zero per-input regressions in all three normalizations, with 12,528
independently checked CSV rows. Five newly generated body images improved in seven raw
input-route pairs. These are known-font synthetic controls, not new independent documents.

The full matrix measures both frozen sources again on Linux/release. It reuses no Mac
predictions. Page30/50/100 and CLI run on separate CI runners, with one inference process
at a time on each runner. Local full attempts were interrupted when storage was exhausted;
their partial output remains in `development.zip` and is not counted as complete evidence.

The actual public Page regression passed its isolated
[Linux Red/Green run](https://github.com/bokuweb/ndlocr-lite-rs/actions/runs/39514635451).
`page-regression-linux-v1.zip` and its separate SHA manifest preserve all raw test/build
output: O failed the body assertion (0 passed / 1 failed / 0 ignored), and FVB passed
(1 passed / 0 failed / 0 ignored). The fixture, test, frozen inference source and bundle
bindings were checked again locally. This evidence does not satisfy the full matrix gate.

## Frozen inputs and sources

`bundle.json` authenticates every archive and member. It contains:

- **921 NN input IDs / 922 scoring IDs / 920 distinct source RGB inputs**. The historical
  duplicate paragraph IDs are retained. The extra Rashomon GT alias does not trigger NN.
- `data.zip`: all encoded input images and literal GT bytes; no font dependency.
- `sources.zip`: baseline Git `1f240be`, the exact isolated FVB source, pinned Cargo.lock
  snapshots, and the frozen official/independent scorers. No model or native executable.
- `development.zip`: raw development Page results, commands, failures, data preparation,
  source provenance, FVA rejection, and FVB preflight metrics.
- `geometry-fva.zip` / `geometry-fvb.zip`: actual Rust decoded/prepared/cropped RGB evidence.
  The FVB 1,836 raw/prepared cases selected 920 crops; independent crop and anchor counts
  agreed, with no annotated glyph loss or promotion of the new portrait-horizontal controls.

Rashomon is one book / one scanned page / three main-text column crops, not three independent
documents. Source: [NDL PID1088339](https://ndlsearch.ndl.go.jp/books/R100000002-I000000570013)
([IIIF manifest](https://dl.ndl.go.jp/api/iiif/1088339/manifest.json), Access Restrictions=PDM).
Ruby pixels are retained while GT measures main-text extraction. The assistant transcribed
and visually checked the crops twice; this is not independent expert annotation. The
ambiguous `剝/剥` glyph is scored against both separate GT variants, requiring nonregression
for both. The earlier clipped trial crops and pre-NN corrections remain in development evidence.

## Reproduction

Integrity checks need Python 3.9+ only:

```sh
python3 -m unittest discover -s tests -p test_vertical_page_matrix.py -v
python3 tools/vertical_page_matrix.py --validate
```

The 14 model-free tests also ensure foreign runner paths are compared lexically,
without resolving `/home/runner/...` on the verification host's filesystem.
The CI guard examines the current push on PR synchronization, because PR path filters
alone include earlier commits. Documentation-only pushes skip the expensive NN jobs.

For NN replay, use Rust stable and the pinned official assets. Each output directory must be
new. Each kind builds both source snapshots, refreshes mtimes after the preceding build,
requires fresh non-test lib/CLI/Page artifacts, freezes executable SHA, and records raw output.
The Page executables must differ. All CLI paths are pinned explicitly to the same assets.

```sh
git fetch --depth=1 origin 1f240be66f71dc1819a99c89095ad9fdf8953695
python3 tools/fetch_upstream_assets.py models
python3 tools/fetch_upstream_assets.py models --offline
python3 tools/vertical_page_matrix.py --kind page30 --output out/page30
python3 tools/vertical_page_matrix.py --kind page50 --output out/page50
python3 tools/vertical_page_matrix.py --kind page100 --output out/page100
python3 tools/vertical_page_matrix.py --kind cli --output out/cli
python3 tools/vertical_page_matrix.py --artifacts out --output verified
```

The scorer requires all four complete jobs and actual executable/model/input command
bindings. It checks 44,256 official CSV rows independently, rejects any per-input character
or word regression, requires a fresh raw body gain, and requires unchanged CLI text.
It does not infer internal forwards from Page calls. Models/fonts/binaries are excluded
from the Git packet; root Cargo.lock remains ignored. The one-off CI workflow uses the
same commands, and retains failures and raw artifacts for review.

## Completed Linux Page kinds; CLI still pending

`completed-pages-linux-v1.zip` preserves Page30/50 from run 39514388558 and all four
Page100 shards from run 39532673209. The compact JSON index and compressed member
index authenticate all 11,249 members. Original Page50's summary had a stale Page30
scope label; its separately corrected label is retained, and model50 CSVs are checked directly.

All **11,052 saved Page results / 33,192 official CSV rows** were revalidated, with
zero per-input character/word regressions in none/width/strict. Raw gains were 35
input/route pairs for model30, 35 for model50, and 8 for model100. Repeated models and
settings must not be added into an independent-image benefit rate. Saved runs loaded
12 PageRecognizers / 24 model sessions. **CLI and the full four-kind gate remain incomplete.**

Replay without weights, fonts or native executables, using a fresh output directory:

```sh
python3 /path/to/repo/docs/benchmarks/vertical-page-fallback/verify_completed_pages.py --output /new/output/directory
```

The verifier authenticates/extracts the packet, validates complete Page source/build/argv,
pre-NN plans and coverage, independently recalculates all CSV rows, and reproduces the
model-choice diagnosis. It always reports `full_gate_pass: false` for this Page-only packet.
The real-packet integration test failed before the verifier existed, then passed.
Pure classification and existing reader contracts also passed (29 tests).

The separate diagnosis classified 424 model/setting/source observations on 99 input IDs /
99 RGB inputs, from 7,368 comparisons. A strong single portrait result from model30/50
beats model100 against every fixed GT alias. Fourteen complete-prefix observations
refer to only three known-font synthetic images. This is a raw-limit proxy and saved
Page comparison, not a direct H/O or internal CLI trace; no selection policy changed.

One successful old CLI command for `advance_bold_71_pad8` produced the same 24-scalar
prefix, whereas saved model30/50 Page results contained 71 scalars. Its source/fresh
build/argv/framing were checked, but its timed-out parent lacks the final materials
stamp. It is kept only under `analysis/incomplete-cli-command`, outside complete
`artifacts` and every adoption gate. Fresh complete CLI results and an internal
cascade trace remain necessary before a new policy is proposed.
