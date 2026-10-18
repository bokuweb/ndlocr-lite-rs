# OCR Evaluation Report

- normalization: `width`
- truth files discovered: `79`

## Aggregate Metrics

| system | matched_files | missing_files | CER | WER |
|---|---:|---:|---:|---:|
| after | 79 | 0 | 0.4449 | 0.8526 |
| before | 79 | 0 | 0.5583 | 0.9629 |

## Notes

- Lower is better for both CER and WER.
- Systems are compared only on files that exist in each prediction directory.
- Use `--require-all` to fail fast when prediction files are missing.

## Missing Files
