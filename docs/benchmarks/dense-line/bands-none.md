# OCR Evaluation Report

- normalization: `none`
- truth files discovered: `79`

## Aggregate Metrics

| system | matched_files | missing_files | CER | WER |
|---|---:|---:|---:|---:|
| after | 79 | 0 | 0.1098 | 0.6479 |
| before | 79 | 0 | 0.2594 | 0.6570 |

## Notes

- Lower is better for both CER and WER.
- Systems are compared only on files that exist in each prediction directory.
- Use `--require-all` to fail fast when prediction files are missing.

## Missing Files
