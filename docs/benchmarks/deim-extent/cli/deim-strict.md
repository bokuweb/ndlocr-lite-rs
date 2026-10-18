# OCR Evaluation Report

- normalization: `strict`
- truth files discovered: `4`

## Aggregate Metrics

| system | matched_files | missing_files | CER | WER |
|---|---:|---:|---:|---:|
| after | 4 | 0 | 0.0357 | 0.4839 |
| before | 4 | 0 | 0.0522 | 0.5000 |

## Notes

- Lower is better for both CER and WER.
- Systems are compared only on files that exist in each prediction directory.
- Use `--require-all` to fail fast when prediction files are missing.

## Missing Files

