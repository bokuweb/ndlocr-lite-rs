# OCR Evaluation Report

- normalization: `none`
- truth files discovered: `79`

## Aggregate Metrics

| system | matched_files | missing_files | CER | WER |
|---|---:|---:|---:|---:|
| after | 79 | 0 | 0.4566 | 0.8656 |
| before | 79 | 0 | 0.5699 | 0.9769 |

## Notes

- Lower is better for both CER and WER.
- Systems are compared only on files that exist in each prediction directory.
- Use `--require-all` to fail fast when prediction files are missing.

## Missing Files
