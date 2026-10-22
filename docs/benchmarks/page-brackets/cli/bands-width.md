# OCR Evaluation Report

- normalization: `width`
- truth files discovered: `5`

## Aggregate Metrics

| system | matched_files | missing_files | CER | WER |
|---|---:|---:|---:|---:|
| after | 5 | 0 | 0.0429 | 0.7249 |
| before | 5 | 0 | 0.0448 | 0.7407 |

## Notes

- Lower is better for both CER and WER.
- Systems are compared only on files that exist in each prediction directory.
- Use `--require-all` to fail fast when prediction files are missing.

## Missing Files

