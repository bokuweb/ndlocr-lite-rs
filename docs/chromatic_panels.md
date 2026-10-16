# Chromatic horizontal panel preparation

## Specification (2026-10-16, before implementation)

The simple threshold detector sees a blue gradient banner as one solid ink band,
then safe line chunking rejects it. Keep the model and recognized text unchanged.
Add a reusable RGB preparation step for broad chromatic horizontal panels:

- Return borrowed input for ordinary achromatic pages; reject inconsistent/overflowing dimensions.
- Identify horizontal spans with substantial chromatic coverage and at least one broad solid-color run. Do not erase isolated colored text or thin colored rules.
- Within each span, estimate each column's background from a majority of chromatic pixels. Normalize darker and lighter deviations to dark-on-white RGB, without changing dimensions or coordinates. Preserve pixels outside qualified panels.
- Use the same prepared pixels for line detection, safe splitting and PARSeq. Do not perform text substitutions or suppress recognition failures.
- Preserve all three existing omo corpora. This is deliberately limited to broad chromatic panels, not arbitrary photographs, grayscale inverse text, tables, or a production-ready page detector.

## Acceptance / next Red

Before implementation, pin a blue gradient banner with white text from omo's existing
public PDF development fixture. Source and crop are recorded beside the PNG. Human
truth is `電子入札運用基準 （受注者様向け）` (including one visible inter-phrase space).
Old behavior fails before inference. Test synthetic gradients, both text polarities,
unchanged gray pages, isolated colored glyphs/rules, invalid lengths and boundary colors.
Then exercise real PARSeq and retain the development/evaluation distinction. Record
latency separately; do not infer page-wide CER from one header.

## Second Red: safe chunk lookahead

Real inference revealed a separate chunking problem: at the 8x aspect limit, the
splitter left the last heading character alone (87x64), reading 準 as 注. Before
changing splitting, pin that orphaned crop in a failing regression. Compute the
minimum feasible chunk count over blank-column boundaries and the existing ink-run
limit; balance only among boundaries that allow the remaining suffix to fit. Keep
word-space preference and all coverage/blank-boundary guarantees. This addresses
image context, not dictionary replacement. Full-width parentheses may still be
recognized as ASCII by the fixed model; count those errors against unchanged truth.

Acceptance keeps the original human truth unchanged. The fixed model still emits
ASCII parentheses: the header threshold is at most two substitutions with equal
character count, and the heading must contain the correct 基準. Those two errors
are reported, not normalized away. The original whole-header failure remains the
before baseline.

## Local validation

The real header now reaches PARSeq as `電子入札運用基準 (受注者様向け)`:
the heading is correct and the two full-width parenthesis substitutions remain.
All eight panel tests (including real inference and two property tests) pass,
as do the repeated-character session/unprocessed regressions. The public API
retains dimensions and positions; omo also exercises the whole-page path.
This is development evidence, not an independent generalization benchmark.

## Third Red: retain word boundaries

The first lookahead implementation increased omo's separate-document errors from
9/91 to 13/91: minimizing chunk count strictly prevented existing word-space
splits. Pin a synthetic word boundary that needs one extra chunk. Use lookahead
to reject impossible suffixes and calculate the balance target, while allowing
the established word-space preference to choose additional feasible chunks.
All existing corpus ceilings remain unchanged.
