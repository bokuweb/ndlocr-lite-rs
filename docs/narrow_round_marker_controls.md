# 狭いround profile用の追加認識前対照

## 生成・NN前仕様

比較元は1ec730a。test-only案は既存137,991保存raw組で製品本文を全件維持し、
新案の45変更がnone/widthの完全一致へ回復、全正規化の文字/単語退行0だった。
これは保存raw比較で新NNではなく、モデル/方向/重複RGBを独立標本と扱わない。
凍結したprofileの数値と本文側条件を変更せず、次の全対照を生成する。

- 本文は既知の `情報` / `報情`。同じ3font fileのface index0、20/28pxを使う。
  FreeType BASICのpalt未指定1 modeと、CoreTextのpalt0/1の2 mode、計3 mode。
  実際のfont/face/path/SHA・glyph・advanceを記録し、FTでnumeric glyph IDやpaltを捏造しない。
- 本文内ASCII空白0/1、端closer `)` / `）`、先頭/末尾、closerと本文間ASCII空白0/1、
  上下紙0/2/10、PNG/JPEG75（subsampling2）の3,456文字画像。
  native画像を保存し、全非白画素のink bboxを原画素のまま2px白紙で囲うROIを基準にする。
  nativeの余白8などをROIへ流用した前段との違いを明記し、内部の実advance/空白は保持する。
- 同じbare本文nativeに、none/1px細線/狭い矩形bar/4×4点/斜線/chevron/左向きcurve/分岐forkを
  先頭/末尾・gap0/1・PNG/JPEG75で加える4,608非文字画像。追加紙なし。
  合計8,064入力、実native描画648回（FT216、CoreText432）。
- 非文字shapeは本文ink高Hを基準とし、細線幅1、bar/斜線/chevron/curve/fork幅はmax(3,ceil(H/6))、
  高さH。点だけ4×4、none幅0。全shapeを本文bboxの外側へ置き、全元非白画素保持と重なり0を検査する。
  2,304 pre-JPEG組を独立再構成する。本文だけの真値を全て保存し、結果で行を除外しない。
- 斜線は左上から右下の連続1px線。chevronは上下端が左で中央1点へ直線的に進む1px線。
  curveは高さの両側floor(H/3)区間で左右を移動し、中央を平らにしたright curveを左右反転する。
  forkは同じright curveの中央行の右端から3px左へ枝を加え、その直前行へ間の画素を加えて8近傍で接続する。
  幅3のときは枝位置を左端0へclampし、間の1pxを加える。native本文高6px未満なら生成を停止する。
  literalなnative `)` と同じ画素をnontextと呼ぶ対照は作らない。
- 各入力の(width,height,rgb SHA)を全137,991親ケースと照合し、全一致元と重複フラグを保存する。
  重複/未重複を別集計し、同じRGBで異なる真値が見つかれば停止して記録する。
- 全tuple/寸法/元RGB・SHA/真値/実native/font/shape元画素/atlasを検査し、72例以上を目視する。
  NN前に真値を仮rawとする幾何到達性を確認するが、OCR精度に数えない。
  未重複RGBに適用可能例が0なら新NNへ進まない。支持条件を生成結果へ合わせ直さない。
- 全入力の元Auto/有効Horizontal tensorを3形状でbit照合してから全ROIを1forwardで認識する。
  Auto回転する原画像も全て元Horizontal指定で測定し、別の独立画像と数えない。
- 旧新/全mode/文字・非文字/重複・未重複/元方向群のnone/width/strict文字・単語を比較する。
  どれかが悪化すれば本案を停止し、同じNN結果へ閾値を合わせ直さない。
  未重複RGBへの実NN適用と全群非悪化が確認されるまで製品接続・精度改善の主張はしない。
