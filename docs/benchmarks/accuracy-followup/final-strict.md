# OCR Evaluation Report

- normalization: `strict`
- truth files discovered: `1`

## Aggregate Metrics

| system | matched_files | missing_files | CER | WER |
|---|---:|---:|---:|---:|
| after | 1 | 0 | 0.0548 | 0.4375 |
| before | 1 | 0 | 0.1765 | 0.4688 |

## Notes

- Lower is better for both CER and WER.
- Systems are compared only on files that exist in each prediction directory.
- Use `--require-all` to fail fast when prediction files are missing.

## Missing Files

