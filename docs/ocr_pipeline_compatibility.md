# OCR pipeline compatibility and reusable line processing

## Specification (2026-10-16, before implementation)

- Track the official Python repository separately from this Rust port. Reviewed upstream:
  `ndl-lab/ndlocr-lite@d001dad2013ceabb5736ff8d637474e443eb73c2` (latest tag 1.3.1).
- PARSeq rotates counterclockwise when `height > width * 0.8`, including nearly square crops.
  Use exact integer comparison to avoid overflow and retain direct-write/scratch API parity.
- Both simple line detectors preserve nearby thin horizontal glyphs at either line edge.
  They do not absorb distant margin noise or shrink the detected band.
- Expose safe horizontal line chunking from the OCR library. Chunks cover the full line
  exactly once, split on blank columns, and reject unsplittable long solid regions.
  Preserve the existing 24x256/30-character model policy and Japanese punctuation spacing.
- omo owns document conversion, request limits and failure policy; it should use these
  generic OCR APIs without duplicating image segmentation heuristics.

## Upstream audit

Baseline for omo models: `7bc4d43e9c0a4dc3cc0d22002b31a75b092d5020` (1.2.1).
The reviewed revision adds PDF/text-layer/Markdown/GUI features, CUDA fallback reporting,
DEIM preprocessing optimization, the PARSeq orientation threshold, and dependency notices.
All four model binaries and `NDLmoji.yaml` are unchanged over that range.
Rust already implements CPU execution and direct float32 DEIM preprocessing; this does
not certify numerical parity with Pillow resizing (Rust uses a different resize path).
The orientation threshold is the directly applicable recognition behavior change.
GUI, searchable PDF and Markdown output are outside this OCR-library change.
Latest dependency notices must accompany newly fetched upstream models, with the original
model/license attribution retained. Model assets remain pinned and hash-verified.

## Validation and next work

First pin rotation boundaries and missing edge pixels in failing tests, then move the
existing chunk regression/property tests to the library. Compare omo fixed corpora
(588 rendered characters, 117 scan characters, 91 development characters).
The colored-header page currently fails before PARSeq because simple thresholding treats
its background as ink. Keep this failure explicit; background-aware preparation or DEIM
detection needs its own regression fixture and must not silently omit the header.

## Updating the official baseline

This repository is a Rust port, not a GitHub fork with shared history. Do not merge Python
commits blindly or mark an entire upstream release as implemented. For each update:

1. Fetch the official repository and record both its latest release and reviewed commit.
   Compare that commit with `revision` in `docs/upstream-assets.json`, especially
   `src/parseq.py`, `src/deim.py`, `src/ocr.py`, `src/model`, `src/config/NDLmoji.yaml`,
   `LICENCE` and `LICENCE_DEPENDENCEIES` (`git diff OLD NEW -- PATHS`).
2. Document each applicable inference change and deferred feature here. Add a failing
   behavioral regression before porting each applicable change. Model changes need
   fixed-image CER and latency comparison before adoption, independently of code changes.
3. Update the manifest's immutable commit URLs and SHA-256 hashes from the reviewed
   checkout, including both license notices. Never change hashes merely to accept a
   failed download. Run `python3 tools/fetch_upstream_assets.py models` then the same
   command with `--offline`; preserve these notices when redistributing the assets.
4. Run default tests, property tests, actual ONNX regressions and omo's three fixed-image
   corpora. This PR adds Linux/macOS real-inference CI to the Rust library.
5. Commit the audit and pin together. Update omo's public git dependency only to the
   validated Rust commit. Keep upstream release features outside this port explicit.

The older `fetch_models_from_ndlocr_release.sh` remains a manual release-archive helper;
use the pinned fetcher above for reproducible tests and reviewed assets. No newer model
was adopted in this update: the four official model SHA-256 hashes are unchanged from
1.2.1. ZIP 2.x was entirely yanked at the time of clean resolution; ZIP 9.0.3 with only
Deflate enabled restores clean builds, covered by the existing DOCX round-trip tests.

Clean ONNX resolution also exposed an incompatible ORT rc.14 API update. Pin ORT
to `=2.0.0-rc.13`, the version already validated by omo, and exercise actual ONNX
compilation/inference in CI before intentionally updating that prerelease API.

## Local validation (2026-10-16)

- Confirmed Red: square-crop rotation, thin leading glyph and missing public chunk API.
- Green: 139 default Rust tests (including properties and DOCX round-trip); 3 real
  ONNX raw/session tests using the hash-verified official model/charset.
- 2 offline asset-fetch regressions; all 7 actual assets downloaded and reverified.
- The three omo corpora are the integration acceptance check; their results are recorded
  in omo. Colored backgrounds remain an explicit, separate pending regression.
