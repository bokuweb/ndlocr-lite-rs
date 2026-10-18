# OCR Evaluation Report

- normalization: `width`
- truth files discovered: `1`

## Aggregate Metrics

| system | matched_files | missing_files | CER | WER |
|---|---:|---:|---:|---:|
| after | 1 | 0 | 0.0319 | 0.8293 |
| before | 1 | 0 | 0.1557 | 0.7561 |

## Notes

- Lower is better for both CER and WER.
- Systems are compared only on files that exist in each prediction directory.
- Use `--require-all` to fail fast when prediction files are missing.

## Missing Files

