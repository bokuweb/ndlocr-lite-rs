# FVA: DEIM欠落時の支持付き単縦列fallback（実装前仕様）

## 狙いと固定範囲

1f240beのOは全444入力・8経路で無回帰だった。P1/P4診断では、DEIMが欠落した注釈付き
111ケース（144既知画像×6経路内の重複回数）がHorizontalの101..146帯に分かれていた。
まずこの単縦列の方向選択を改善する。原文の文字置換、H/Oの条件緩和、外部dictの内蔵はしない。
rootの推論codeは変更せず、1f240beの隔離コピーでRed→Green→Refactorを進める。

## 方向選択の事前条件

- 呼び出すのはDEIMの有効行矩形が0の場合だけ。DEIM経由の認識は変更しない。
  既存fallbackと同じ背景準備後RGBを読み、元API入力bufferを変更しない。
- 画像の幅は12px以上、height>=8*width。RGB長と積overflow、不正寸法を検証する。
- 既存detect_vertical_textline_columns(threshold220)がちょうど1列を返し、その幅が
  画像幅の半分以上であること。既知縦書き向けAPIを無条件に自動適用しない。
- RGBの全非白画素（sumRGB<765）を覆う半開envelopeを求め、画像内で1pxだけ広げる。
  この領域のwidth>=12、height>=8*widthを要求する。軽いAAを除いて字形欠損を隠さない。
- この領域を既存の無補間90度回転で支持判定する。既存ink-run走査を小さな内部helperで共有し、
  threshold220のインク列数が長軸の1/3以上、横断方向extentが短軸の1/2以上であること。
  runの長さと横断extentがともに短軸の3/5以上のanchorを6つ以上要求する。
  方向判定では、2字以上の横書き行を単縦列にしないため境界保護Mより強いsquare支持を使う。
- 回転後の領域に既存band_text_supportを適用し、塗り四角・細線だけを文字支持としない。
  どれかが不成立なら既存水平fallbackをそのまま使う。未検出を成功扱いで空にしない。
- 幾何成立時だけ1領域をFragment+Autoで試す。結果が非空かつfiniteで、confidenceが
  max(0.9,通常min_line_confidence)以上の場合だけ採用する。成功した結果を再推論しない。
  認識失敗・低confidence・空文字なら既存水平fallbackを実行し、元経路を空に置き換えない。
  水平fallbackはHorizontalのまま、既存filterとH/O自体は変更しない。

これは単縦列を支持する最初の方針であり、余白が大きい列・複数列・混在方向全体の自動判定を
保証しない。未知入力に安全と仮定せず、反例を保存して採用gateで判定する。

## 先行テストと幾何検査

- 純粋な単縦列positiveをRedにし、入力byte/元座標/全非白画素保持を検査する。
- 横長行、tall portrait横書きword行、2列（横gap/縦ずれ）、白紙、塗り図形、細線、
  anchor5/6・比率/密度/12px境界・不正RGBを例/PBTで固定する。
- helperは新しい方向選択だけに使い、既存公開splitter/H/Oの結果を変えない。
- NN前に既知Page444と既知native414の和集合、および新規96画像を除外せず走査する。提案領域の元glyph BBox/全画素を検査し、
  字形を切る候補が1つでも残れば候補全体をNN前に保留する。未注釈は別扱いにする。
  幾何成功を精度成功とは扱わない。

## 未測定の新規96 controls（NN前に固定）

- 本文60: 既知3font、(font28/advance32, font40/advance48)、padding0/12、
  長さ8/12/31/51/101。幅font_size+8+2padding、上余白8+padding、下余白8+padding。
  一つの新しい長文の先頭N scalarでGTを作り、全条件と全文をrender前に保存する。
- 図形24: 背景white/warm/gray、filled_square/zigzag/thin_bar/brush、group8/40。
  empty GTは従来どおり「描いた非文字」の診断規約と明記する。
- portrait横書き12: 同じ3font×2size×2padding、8行の複数字word。
  canvasの高さを幅の10倍以上にして、単にportrait比で方向を決める反例にする。
- font/PNG/RGB/GT/字形BBoxとgeneratorのSHA、独立getmask配置と全非白画素を保存する。
  既知全画像と新規PNG/RGBの重複を調べ、生成後に条件/GT/閾値を変更しない。

## 認識・採用gate

- baselineと候補を同じ固定input和集合へ、Page APIの全3model×eval/defaultと
  CLI DEIM/bandsで比較する。旧444 baselineは推論sourceが同じことを証明した場合だけ再利用する。
- 実行物/source/model/charset/lockとコマンドを固定する。コピーのmtimeによるstale cacheを防ぎ、
  fresh=false・候補とbaselineの異なるSHA・実際のpositive呼出しを検証する。
- 全入力/各経路/none,width,strictの文字・語誤りを公式と独立距離で照合し、どれかの悪化を拒否する。
  新規の注釈付き本文で少なくとも1件、文字誤りを減らして語誤りを悪化させないことを必要とする。
  図形だけの減少や既知111ケースだけの改善では採用しない。
- GT/rawをstripせず、CLI transport LFを1つだけ区別する。原出力/全失敗/未達条件を保存する。
  未計測の内部forward/loadはnullにし、計測したNN/外側API/再利用/モデルloadを区別する。
- geometryとモデルgate、既存回帰とCIが通るまでrootへ接続しない。目標全体は継続する。

既知範囲の補足（追加NN前）: native414の原入力にもPageとしての方向選択を検査する。
Page444と同一ID/RGB/寸法/GTの行だけを統合し、別名・異なるRGB/GTは保持する。
最終ID数とunique RGB数をmanifestで先に固定し、重複を独立標本や新規NN呼数に数えない。

固定予定件数: 既知和集合714 ID（713 unique RGB、同一IDの144行を統合）、
新規96を合わせ810 ID/809 unique RGBを予定する。nativeの全270注釈が和集合へ保存された。

幾何記録の補足（追加NN前）: 選択された領域のRust実decode RGBと、既存背景準備で変わったRGBを
可逆ZIPへ保存して独立PIL cropと照合する。JPEGのdecode実装差を画素一致と仮定せず、差を報告する。
新規PNGのdecodeは生成RGBと一致を要求する。元の字形BBoxは変更しない。

実モデルの先行棄却試験: 全新規96とP1既知144、計240入力を全6 Page API経路で比較する。
新規baseline96を測り、既知144は同じ固定O実行物の保存結果を使う。
ここで1入力/正規化でも退行が出たら全候補を保留し、条件を結果に合わせて緩めない。
先行成功だけで採用せず、採用には引き続き全810/8経路と新規本文positiveを必要とする。

実行物gateの範囲補足（FVA追加NN前）: 新規Page API実行物は必ずfresh=falseとbaselineとの差分SHAを要求する。CLIは別のCachedPageRecognizerを使い、FVAのPageRecognizer接続点を通らない。CLIが同一SHAの場合はsource/link依存と実行物の同一性を証明して既存444結果を再利用できるが、新規366入力の両CLI経路は実測する。同一CLIを理由にPage APIのfreshness条件を省略しない。
