# OCR Evaluation Report

- normalization: `width`
- truth files discovered: `46`

## Aggregate Metrics

| system | matched_files | missing_files | CER | WER |
|---|---:|---:|---:|---:|
| after | 46 | 0 | 0.0916 | 0.6258 |
| before | 46 | 0 | 0.1307 | 0.6476 |

## Notes

- Lower is better for both CER and WER.
- Systems are compared only on files that exist in each prediction directory.
- Use `--require-all` to fail fast when prediction files are missing.

## Missing Files
