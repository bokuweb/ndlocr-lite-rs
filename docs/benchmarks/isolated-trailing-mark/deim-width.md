# OCR Evaluation Report

- normalization: `width`
- truth files discovered: `103`

## Aggregate Metrics

| system | matched_files | missing_files | CER | WER |
|---|---:|---:|---:|---:|
| after | 103 | 0 | 0.2019 | 0.5738 |
| before | 103 | 0 | 0.2183 | 0.5745 |

## Notes

- Lower is better for both CER and WER.
- Systems are compared only on files that exist in each prediction directory.
- Use `--require-all` to fail fast when prediction files are missing.

## Missing Files
