# Morphological OCR correction

## Scope

Morphological correction is an optional post-processing step. It is enabled only
when the caller provides a morphological dictionary path. The default pipeline
must keep the current text unchanged.

The first implementation uses `delarocha` with a Vibrato `system.dic` or
`system.dic.zst` dictionary. It tokenizes each recognized line, finds tokens
reported as unknown by the dictionary, and tries small OCR-confusion edits only
inside those unknown spans.

## Correction policy

- Apply only generic OCR confusion pairs, such as visually similar kana, kanji,
  and symbols.
- Replace a span only when the replacement candidate is accepted by the
  morphological dictionary as known text.
- Keep all document-type-specific corrections in `--post-dict` or
  `--rule-pack`, not in the core correction table.
- Keep correction optional and conservative. When no candidate is known, return
  the original text.
- Support both one-character substitutions and short span collapses for generic
  glyph confusions. For example, `禾リ` may be proposed as `利`, but the
  replacement is applied only if the whole candidate token is known by the
  morphological dictionary.

## CLI

`recognize-page` accepts:

- `--morph-correct-dict <PATH>`: enable morphological correction using a
  Vibrato-compatible dictionary file.

The correction runs after `--post-dict` and before reading-order structural
rules so later line-level cleanup sees the corrected text.
