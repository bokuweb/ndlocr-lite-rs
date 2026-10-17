# Repeated text fixtures

Synthetic text rendered in Arial at 42 px; no font files are distributed.
Expected text was authored independently from recognition output.
The PNGs are cropped to line bounds with one white row and two white columns
around the ink, matching the existing naive line detector used by omo.
`repeated_digits`: 111111; `repeated_pattern`: 123123123.

The legacy API removes repetitions. The unprocessed API must preserve the decoded
text, and the legacy APIs must retain their previous sanitized behavior.
