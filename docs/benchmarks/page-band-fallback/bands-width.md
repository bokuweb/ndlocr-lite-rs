# OCR Evaluation Report

- normalization: `width`
- truth files discovered: `127`

## Aggregate Metrics

| system | matched_files | missing_files | CER | WER |
|---|---:|---:|---:|---:|
| after | 127 | 0 | 0.0824 | 0.4471 |
| before | 127 | 0 | 0.0847 | 0.4553 |

## Notes

- Lower is better for both CER and WER.
- Systems are compared only on files that exist in each prediction directory.
- Use `--require-all` to fail fast when prediction files are missing.

## Missing Files
