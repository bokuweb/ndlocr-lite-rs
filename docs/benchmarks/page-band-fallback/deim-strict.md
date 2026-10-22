# OCR Evaluation Report

- normalization: `strict`
- truth files discovered: `127`

## Aggregate Metrics

| system | matched_files | missing_files | CER | WER |
|---|---:|---:|---:|---:|
| after | 127 | 0 | 0.1972 | 0.5512 |
| before | 127 | 0 | 0.1992 | 0.5534 |

## Notes

- Lower is better for both CER and WER.
- Systems are compared only on files that exist in each prediction directory.
- Use `--require-all` to fail fast when prediction files are missing.

## Missing Files
