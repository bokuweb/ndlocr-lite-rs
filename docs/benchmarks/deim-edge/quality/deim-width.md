# OCR Evaluation Report

- normalization: `width`
- truth files discovered: `15`

## Aggregate Metrics

| system | matched_files | missing_files | CER | WER |
|---|---:|---:|---:|---:|
| after | 15 | 0 | 0.0552 | 0.7337 |
| before | 15 | 0 | 0.0556 | 0.7372 |

## Notes

- Lower is better for both CER and WER.
- Systems are compared only on files that exist in each prediction directory.
- Use `--require-all` to fail fast when prediction files are missing.

## Missing Files

