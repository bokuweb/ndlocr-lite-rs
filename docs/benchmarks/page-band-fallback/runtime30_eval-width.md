# OCR Evaluation Report

- normalization: `width`
- truth files discovered: `127`

## Aggregate Metrics

| system | matched_files | missing_files | CER | WER |
|---|---:|---:|---:|---:|
| after | 127 | 0 | 0.3980 | 0.7051 |
| before | 127 | 0 | 0.4963 | 0.7816 |

## Notes

- Lower is better for both CER and WER.
- Systems are compared only on files that exist in each prediction directory.
- Use `--require-all` to fail fast when prediction files are missing.

## Missing Files
