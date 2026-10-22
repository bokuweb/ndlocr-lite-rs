# OCR Evaluation Report

- normalization: `none`
- truth files discovered: `20`

## Aggregate Metrics

| system | matched_files | missing_files | CER | WER |
|---|---:|---:|---:|---:|
| before | 20 | 0 | 0.0797 | 0.7950 |
| after | 20 | 0 | 0.0801 | 0.8003 |

## Notes

- Lower is better for both CER and WER.
- Systems are compared only on files that exist in each prediction directory.
- Use `--require-all` to fail fast when prediction files are missing.

## Missing Files
