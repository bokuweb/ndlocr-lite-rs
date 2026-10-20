# 点付きLatin候補の追加対照

## 生成・追加NN前の仕様

比較元はf31a87d（e42eb19と同じ製品Rust）。候補はtest-onlyで、ownerのSHAは
6386307740cd222df614960fb56c47c4485db726b195e1fa81d5a53e02bae495、
policy prefixのSHAはb2c3240fa2ff7b58718e301f643cc56615ff780012fb48dfb72817dad9369a26。
30純粋テスト/PBT後に固定した。全223,143保存rawの実Rust比較では12組が変わり、
11組がnone/width完全一致、別の1組は既存i/j誤読を残して括弧だけが改善した。
全群/正規化の入力別文字・単語退行0。元RGBは既知4種類で、まだ追加NNや製品接続はない。

この先の結果へ候補条件を合わせ直さない。生成前に以下の全tuple、source、font、
rendererとpolicyのSHAを保存する。画像・raw・棄却例・失敗ログを保持する。

## 全19,440条件

- engineはFreeType BASIC/palt未指定、CoreText/palt0、CoreText/palt1の3モード。
  既存と同じ3font file/face0（maru/gothic/mincho）、20/32pxを使う。
  font file SHA、実際のglyph/advance/配置、描画サイズ、nativeとcrop済みRGBのSHAを保存する。
  FreeTypeにpaltやnumeric glyph IDを捏造しない。CoreTextはfallback/欠字/別font runを拒否する。
- 本文は Abjp / Ajbp / Bpjg / Adip / Aimq / Apji の6種類。
  jのみ・iのみ・i/j両方、ascender/descenderと位置の対照である。
  既知fontでの新しい開発条件で、未知font/実資料のholdoutとは扱わない。
- textは本文内ASCII空白0/1（全字間）、round closer半角/全角、前/後、
  closerとのASCII空白0/1、上下紙0/2/10px、PNG/JPEG75を全交差し、10,368条件とする。
- noiseは空白なしの全6本文と、全字間空白ありのApjiだけ（計7本文形）。
  none/thin/bar/dot/diagonal/chevron/left_curve/fork_inner/fork_outer、前/後、
  gap0/1、PNG/JPEG75を全交差し、9,072条件とする。
  body-only真値は診断上の規約で、chevron/curve/forkの意味を一意な非文字と扱わない。

nativeはtext1,728とbare126の計1,854描画。別に全1,854件を再描画し、実font/glyph/advanceと
RGBを照合する。codec条件は同じpre-codec画像から作り、JPEGはquality75/subsampling2で一度decodeし、
canonical lossless PNG/RGBを保存する。NNと後のCLIは同じdecode済みRGBを使う。

## 図形対照の基準高

前段Latin対照と違い、図形サイズの基準はbare本文の**最大8近傍連結成分高**とする。
上点を含む全前景bbox高とは区別する。これはmarkerを除く候補のstem高へ近い図形も検査し、
上点のために図形だけが大きくなって適用外になる問題を避けるためである。
文字ラベル/モデル出力を参照せず、RGB和660未満の全成分から求める。島も捨てない。

- サイズと画素集合は既存graphicsの9図形をその基準高へ適用する。
  noneは0×0、dotは4×4、他の高さはceil(4H/3)。thinの幅は1、他の幅はmax(3,ceil(H/4))。
  shapeの離散画素集合は前段と同じで、出力を見て変えない。
- 図形はbare本文の全前景bboxの縦中心へ揃える。全非白画素bboxを使って前/後へ配置し、
  必要な上下canvasを広げ、AAを含む全元非白画素を保持する。重なりは0。
- 最大成分高と全前景高が違う例を含む、先行Red/Greenで基準高・配置・近白画素保持を検査する。
  4,536 pre-JPEG組を独立に再構成し、全元RGBの保持と非重複を確認する。

## NN前の確認と採点

- tupleの一意性/全交差/件数を描画前のテストで確認する。既存223,143組とのRGB重複と
  新対照内部の重複/真値競合を全件記録し、同じ元画素を独立標本と数えない。
- engine×font×sizeの18群から各6例を固定して108例を目視する：
  Abjp/compact/半角suffix/外空白0/紙0/PNG、Adip/compact/全角prefix/外空白0/紙10/JPEG、
  Apji/字間空白1/半角suffix/外空白1/紙2/PNG、Bpjg/chevron/後/gap0/JPEG、
  Aimq/fork_inner/前/gap1/PNG、Apji/字間空白1/left_curve/後/gap1/PNG。
- 全元Autoと、有効なHorizontal入力の前処理tensorを3モデル形状でbit照合する。
  Auto回転を除外しない。元RGBでAutoが回転する全条件には、同じ原Horizontalも全3モデルで追加保存する。
- 新rawは全19,440条件×30/50/100の58,320 forwardと、上記元Horizontal全件。
  model/charset/source/binary/input SHAを前後で照合し、実入力captureも照合する。
  幾何到達性（仮の正読rawによる適用検査）は精度測定に数えない。
- 既存と追加全件のnone/width/strictを正式scorerで採点し、文字/単語の入力別退行があれば
  採用を止める。text/noise・engine・font・本文/空白等の群と、全棄却/改善例を保持する。
  未重複RGBの実NN適用も確認する。これだけで未知資料への改善率と主張しない。
- 条件を通過しても接続前の公開Session/Pool Redを先行する。接続後の実製品全件再生、
  選択全件と固定noise対照の公開API/CLIでraw/本文/mean bitを検査し、必要な通常/native/CIを完了する。
  CLI stdoutは末尾LFを正確に1個だけ除く。新規ページ精度測定は別で、既存ページ回帰と区別する。
