# 印字された閉じ括弧の固定開発対照

`corpus.zip` は320画像と対応するliteral truthの計640 memberを含む。
OCR前に `generate.py` で80文字列×角ゴシック/明朝×PNG/JPEG75を固定した。
5括弧種の前/後/内部/対応した括弧、単独・2個連続と、削除対象でない句読点/括弧の対照を含む。
28px・白紙余白8px。truthは描画した文字列そのもので、認識結果を見て変更しない。

`manifest.json` は画像・truth・archive・generator・ローカルfontのSHA、Pillow版と寸法を記録する。
font自体は再配布しない。再生成には記録したmacOSフォントとPillowが必要だが、通常テストには不要。
Rustの通常テストは独立に組み立てた80文字列、全組合せ、画像decode・寸法と全memberを検査する。
評価準備では全SHAも照合する。ZIP SHAは
`deff96e5762ff0d8223cdeefbf9f20e1ceddaef315c3d885927467f29d9b58bb`。

既知フォントの合成開発対照で、未知実資料のholdoutではない。
診断と採否は `docs/literal_closers_probe.md`、`docs/literal_closers.md` と
`docs/benchmarks/literal-closers/` に保存する。
