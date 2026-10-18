# OCR Evaluation Report

- normalization: `none`
- truth files discovered: `103`

## Aggregate Metrics

| system | matched_files | missing_files | CER | WER |
|---|---:|---:|---:|---:|
| after | 103 | 0 | 0.2140 | 0.6060 |
| before | 103 | 0 | 0.2303 | 0.6067 |

## Notes

- Lower is better for both CER and WER.
- Systems are compared only on files that exist in each prediction directory.
- Use `--require-all` to fail fast when prediction files are missing.

## Missing Files
