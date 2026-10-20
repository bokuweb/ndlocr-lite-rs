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
