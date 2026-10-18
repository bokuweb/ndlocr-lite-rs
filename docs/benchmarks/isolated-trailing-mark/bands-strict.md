# OCR Evaluation Report

- normalization: `strict`
- truth files discovered: `103`

## Aggregate Metrics

| system | matched_files | missing_files | CER | WER |
|---|---:|---:|---:|---:|
| after | 103 | 0 | 0.0793 | 0.4339 |
| before | 103 | 0 | 0.1195 | 0.4478 |

## Notes

- Lower is better for both CER and WER.
- Systems are compared only on files that exist in each prediction directory.
- Use `--require-all` to fail fast when prediction files are missing.

## Missing Files
