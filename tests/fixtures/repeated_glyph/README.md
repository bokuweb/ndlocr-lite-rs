# 反復文字の固定開発対照

`corpus.zip` は160画像と対応するliteral truthの計320memberを含む。
認識前に `generate.py` で全組合せを固定した。8単位（1/0/8/I/l/O/一/A1）、
反復数2/4/6/8/12、角ゴシック/明朝、PNG/JPEG75、28px・余白8pxを使う。
真値はその文字単位の反復そのもので、認識結果に合わせて変更しない。

`manifest.json` は画像・真値・archive・generator・使用したローカルfontのSHA、
Pillow版と寸法を記録する。font自体は再配布しない。
再生成にはmanifestに記録したmacOSフォントとPillowが必要で、通常のテストには不要。
Rustの通常テストは全組合せ、literal truth、画像decode、寸法、全memberの構成を検証する。
評価準備では各SHAも照合する。

これは既知フォントによる合成開発対照であり、未知の実文書holdoutではない。
診断と採否の全記録は `docs/benchmarks/visible-repetitions/` に保存する。
