# OCR Evaluation Report

- normalization: `strict`
- truth files discovered: `20`

## Aggregate Metrics

| system | matched_files | missing_files | CER | WER |
|---|---:|---:|---:|---:|
| cascade | 20 | 0 | 0.0414 | 0.4885 |
| fixed100 | 20 | 0 | 0.0425 | 0.4425 |

## Notes

- Lower is better for both CER and WER.
- Systems are compared only on files that exist in each prediction directory.
- Use `--require-all` to fail fast when prediction files are missing.

## Missing Files
