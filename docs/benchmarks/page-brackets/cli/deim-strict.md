# OCR Evaluation Report

- normalization: `strict`
- truth files discovered: `5`

## Aggregate Metrics

| system | matched_files | missing_files | CER | WER |
|---|---:|---:|---:|---:|
| after | 5 | 0 | 0.0341 | 0.4828 |
| before | 5 | 0 | 0.0348 | 0.4943 |

## Notes

- Lower is better for both CER and WER.
- Systems are compared only on files that exist in each prediction directory.
- Use `--require-all` to fail fast when prediction files are missing.

## Missing Files

