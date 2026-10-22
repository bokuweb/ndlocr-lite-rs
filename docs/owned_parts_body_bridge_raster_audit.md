# 接触部品fixtureの描画器を確認する

## 監査実行前仕様

元fixtureの生成源は `tests/fixtures/closer_geometry/generate.py` で、
`ImageFont.truetype` / `ImageDraw.text` を使う。今回の追加10,944画像と16画像screenはCoreTextで、
真値を仮rawとした増分適用が0だった。支持条件を変更せず、描画器の差を確認する。

- 元manifestのfont file/SHA/Pillow versionを照合し、同じmincho file・face index 0・20pxで
  元の `情報」` を同じgetbbox/余白8の式からメモリ上で再描画する。
  既存fixtureの元PNGと、寸法とRGB bytesが一致するか確認する。fixture自体を変更しない。
- 実FreeTypeFontのfamily/style/layout engine、Pillow/FreeType/RAQMの実行時情報、
  全文字とprefixのgetlengthを記録する。BASICはpalt制御を実行できないので、palt=0/1を偽装しない。
- 同じfont fileのCoreText/palt0による同じ文字列も診断用に描画し、font face/advance/元画素を記録する。
  余白の差を除いたink bbox内のRGB/閾値maskの寸法とSHAも記録する。
- 真値を仮rawとして両画像を凍結fc5/固定案へ渡す幾何監査を別途行う。
  OCRの新NN、精度、未知の独立した標本とは数えない。
- 元と同じFreeType描画での追加対照が必要なら、paltを含まない別の生成前仕様へ固定し、
  source/face/layout/実測advanceと字形maskを保存する。既存CoreText入力は全件保持する。
