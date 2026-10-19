# 既知の接触部品へFreeTypeで条件・負例を追加する

## 生成・追加認識前仕様

FreeType screenの10画像でも真値を仮rawとした増分適用が0だったため、そのscreenを停止した。
NNは未実行で全入力と結果を保持する。支持条件は変更せず、既知の対象とその逆順を使い、
対象の字形で非文字ノイズを誤救済しないかを直接検査する。未知字形の汎化とは扱わない。

- 本文は `情報` / `報情`。元のglyphは既知で、一部の原RGBは既存評価と重複する。
  新データの各入力を、旧115,023組の `(width,height,rgb_sha256)` と照合して全一致元を記録する。
  全入力を保持して測定し、既知RGB/未重複RGBの結果を分ける。条件/model別の出力を独立標本と扱わない。
- 同じ3font file、face index0、20/32px、明示FreeType `Layout.BASIC` を使う。
  RAQM/paltは使えないためpalt軸を置かず、全recordで未指定と明記する。
  元のImageFont.getbbox/余白8/ImageDraw.textを使い、字形・advance・橋を編集しない。
- 本文内ASCII空白0/1/2、closer5種類・先頭/末尾、closerと本文間ASCII空白0/1/2、
  上下紙0/1/8、PNG/JPEG75で6,480画像。紙1は元と近い可視サイズの未重複条件も検査するために先行固定する。
- 本文内空白も含む同じ本文のnative画像に、本文だけ/細縦線1px/幅ceil(ink高/4)縦線/4×4点を、
  先頭/末尾・gap0/1・PNG/JPEG75で加える1,152非文字負例。負例に上下紙を追加しない。
  合計7,632入力、実Pillow描画1,116記録。非文字合成だけはink bbox+2pxのcropで元画素を保持し、
  shapeとの重なり0を確認する。text入力は元の余白8/native advanceを保持する。
- 実font face/layout/path/size/SHA、getlength/getbboxと各文字の実glyph maskを保存し、未定義glyphを拒否する。
  numeric glyph IDを捏造せず、Pillowの公開する実bitmap/advanceの検査であることを明記する。
  全tuple/SHA/真値/元RGB、576 pre-JPEG負例を独立に再構成し、72例以上を目視する。
- 追加NN前に全入力を真値の仮rawで凍結fc5/固定案へ渡し、全体と未重複RGBの幾何到達性を記録する。
  OCR精度やNNの改善数には数えない。未重複RGBに増分適用が0ならNNへ進まない。
- 全入力の元Auto/有効Horizontal tensorを3形状でbit照合してから全ROIを1forwardで認識する。
  Auto回転した元画像もすべてHorizontal指定で測定し、方向条件を独立画像へ数えない。
- 旧新全群・既知/未重複各群のnone/width/strictで文字/単語指標を比較する。
  1指標でも悪化すれば案の接続を止め、反例を保存する。同じ結果へ閾値を合わせ直さない。
  未重複RGBに新NNで実際の適用がない場合も、そのまま採用しない。
- 通過時だけ公開接続仕様、実SessionのRed→接続→Green、全製品再生、全選択/境界API/CLI、
  全スイートとfragment page source不変性、独立復元へ進む。支持処理の追加NNは0を維持する。
