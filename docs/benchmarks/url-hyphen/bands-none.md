# OCR Evaluation Report

- normalization: `none`
- truth files discovered: `20`

## Aggregate Metrics

| system | matched_files | missing_files | CER | WER |
|---|---:|---:|---:|---:|
| after | 20 | 0 | 0.0803 | 0.7831 |
| before | 20 | 0 | 0.0813 | 0.7778 |

## Notes

- Lower is better for both CER and WER.
- Systems are compared only on files that exist in each prediction directory.
- Use `--require-all` to fail fast when prediction files are missing.

## Missing Files
