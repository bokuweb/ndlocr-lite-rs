# OCR Evaluation Report

- normalization: `strict`
- truth files discovered: `15`

## Aggregate Metrics

| system | matched_files | missing_files | CER | WER |
|---|---:|---:|---:|---:|
| after | 15 | 0 | 0.0454 | 0.5096 |
| before | 15 | 0 | 0.0457 | 0.5096 |

## Notes

- Lower is better for both CER and WER.
- Systems are compared only on files that exist in each prediction directory.
- Use `--require-all` to fail fast when prediction files are missing.

## Missing Files

