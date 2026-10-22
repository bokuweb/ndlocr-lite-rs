# OCR Evaluation Report

- normalization: `width`
- truth files discovered: `20`

## Aggregate Metrics

| system | matched_files | missing_files | CER | WER |
|---|---:|---:|---:|---:|
| after | 20 | 0 | 0.0593 | 0.7222 |
| before | 20 | 0 | 0.0596 | 0.7249 |

## Notes

- Lower is better for both CER and WER.
- Systems are compared only on files that exist in each prediction directory.
- Use `--require-all` to fail fast when prediction files are missing.

## Missing Files
