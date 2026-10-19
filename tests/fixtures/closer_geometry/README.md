# 閉じ括弧の高さ・複数成分と空白の追加開発対照

OCR前に80文字列×丸ゴシックW4/明朝×20/40px×PNG/JPEG75の640画像を固定した。
5括弧種、Agjp/2026/情報の前/後/内部、和文balanced、単独/連続、空白を含む20対照。
さらに文字を描かない8図形×2形式の16対照を加え、合計656画像と対応truthの1312 memberを
`corpus.zip` に保存した。文字truthは描画したliteral文字列、図形truthは空文字。
余白8px。OCR後に画像/文字列/条件を変更しない。

`manifest.json` はPillow/font/generator/画像/truth/archive SHAと寸法を記録する。
font自体は配布しない。再生成には記録したmacOSフォントとPillowが必要だが、通常テストには不要。
Rustの通常fixtureテストは別に組み立てた全80文字列、8図形、全組合せ・member・truth・decode・寸法を検査する。
archive SHAは `f6a34b4c5caf56a9ea748f41ef362003eef80c44d1c276e7f8d50a350eabbc64`。

既知フォントの合成開発対照で、未知実資料のholdoutではない。
先行仕様は `docs/closer_geometry_probe.md`、全結果は `docs/benchmarks/closer-geometry-probe/`。
