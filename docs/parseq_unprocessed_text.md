# PARSeq decoded text without heuristic replacement

The scored RGB API currently sanitizes decoded text. This can remove legitimate
repeated digits, repeated substrings, closing brackets and some Japanese phrases.

Add `recognize_rgb_u8_with_score_unprocessed`, returning decoder text and mean
confidence before `sanitize_recognized_text`. Preserve the behavior of all existing
recognition APIs and explicit sanitization. Share model loading, RGB preprocessing,
inference and decoding. Preserve existing EOS and low-confidence stopping policies;
this API does not guarantee that inference itself recognizes every character.
Missing files, malformed models and a disabled ONNX feature remain errors.

Regression fixtures independently expect `111111` and `123123123`. The ignored
real inference test requires `NDLOCR_TEST_PARSEQ_MODEL` and `NDLOCR_TEST_CHARSET`;
missing configuration must fail. Default tests cover feature-disabled errors and
existing sanitizer regressions must continue to pass.
