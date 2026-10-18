# 製品経路から取得した原RGB chunk

出典は総務省統計局 *Statistical Handbook of Japan 2026* の物理3ページ目。
`source.json` に元PDF/ページ画像/PNG/生RGBのSHA256、製品コード、取得元の証拠commit、chunk番号、寸法、正解、元の生認識を記録する。日本語PDFのメタデータも元corpusの記録として含むが、この6件のchunkは英語ページのみ。

omo PR #79 の `docs/benchmarks/ocr-2026-10-18-margin-sensitivity/capture.py` で元の全ページ呼出し・本文・座標・信頼度を再現してから、生RGBを保存した。JPEGを後から再cropしたものではない。PNGへの変換前後でRGB完全一致を確認した。

`modern_200` / `modern_150` は余分なcommaが消えた既知の入力、`comments` / `japan_we` は余白縮小で余分なperiodが出た既知の入力。`culture` / `sources` は正しいperiodを保持する対照。6件とも既存資料から選んだ診断用ケースで、新しいholdoutではない。

`cargo test --test margin_sensitivity_tests` は全非白RGB保持とモデル前処理の差を検査する。ONNX比較の再現方法は `docs/margin_sensitivity.md` を参照。
