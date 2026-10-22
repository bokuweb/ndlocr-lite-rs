# OCR Evaluation Report

- normalization: `strict`
- truth files discovered: `20`

## Aggregate Metrics

| system | matched_files | missing_files | CER | WER |
|---|---:|---:|---:|---:|
| before | 20 | 0 | 0.0425 | 0.4425 |
| after | 20 | 0 | 0.0431 | 0.4799 |

## Notes

- Lower is better for both CER and WER.
- Systems are compared only on files that exist in each prediction directory.
- Use `--require-all` to fail fast when prediction files are missing.

## Missing Files
