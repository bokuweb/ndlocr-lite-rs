# AGENTS.md

Agents working on this repository MUST follow the rules below.

## Development principles

1. **Test / document first**
   - Before implementation, write the specification for the target phase under `docs/`.
   - Before implementation, add tests first (failing tests).
   - Implement with minimal changes so the new tests pass.

2. **t_wada-style TDD**
   - Strictly follow Red → Green → Refactor.
   - Do not build large chunks at once; iterate in small behavior-sized steps.
   - Make tests serve as specification: expectations and intent must be clear.

3. **Property-based testing**
   - For logic with many boundary cases or input variations, use property-based testing when it pays off.
   - Combine with example tests to pin regression-prone cases.

## Current priority phase

- Follow `docs/plan.md` and implement sequentially from Phase 1.

## Post-processing and domain correction

- **Core (`src/postprocess/page_rules.rs`, etc.)** must not accumulate ad-hoc fixes specialized to statutes or contracts. Keep it to generic structural cleanup.
- **Per-document-type replacements and adjacent-line merging** are supplied via `--post-dict` or **`--rule-pack` (external YAML)**. Examples: `docs/rule_pack.example.yaml` / `docs/rule_pack.scaned0.yaml`.
- For quantitative accuracy comparison, use `docs/evaluation.md` and `tools/eval_ocr_metrics.py`; for in-process Rust A/B, use `tools/run_eval_rust_ab.sh`. Evaluation data is under `tests/fixtures/eval/` (see `tests/fixtures/eval/README.md`).
