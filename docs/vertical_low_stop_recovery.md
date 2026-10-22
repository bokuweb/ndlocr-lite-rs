# S: H後のLowConfidence停止を細かい二分割合意で再検査（実装・新対照・追加NN前仕様）

Qは新72 nativeの全10,368文字/語比較で変更0のため保留した。R0も主/副spanの合意から新規補完を
支持しなかった。S0では既知1組の副/quarter spanが同じ50 scalarを返し、現在H結果43 scalarを
残した7文字挿入が一意になった。全12部分はEOS・finite0.9..1・31未満だった。
旧H primaryのRGB/境界、連結本文/加重mean bitsを再構成すると現在H結果と一致し、その一部分だけが
LowConfidenceで止まっていた。これは既知合成1組の診断であり、未知資料改善とはしない。

## 対象と停止metadata

- 本体O/1f240beの別コピーを使う。保留Qのhookは含めない。Hの採用/不採用条件、旧O combine、
  通常SingleLine・raw Session・明示Horizontalの認識方針を変えない。
- Hが採用したFragment+Autoだけを対象にする。選択outputの静的T/Cとcharset、元rawと全H partsの
  実T一致を必要とし、元H partの少なくとも一つが実LowConfidence停止だった場合だけ新検査へ進む。
  不明shape/不一致/metadata不在は元H結果を保つ。
- 既存の単一デコーダobserverから停止理由だけを返すcrate-private wrapperを追加する。
  同じlogitsを同じ演算順序で一度だけdecodeし、元の本文/mean bitsとエラーを保つ。
  停止しなければTimestepsExhausted、EOS/LowConfidenceは既存enumを返す。閾値0.35等を変更しない。
  Hのcallbackは同じ借用Session/同じNN出力からmetadataを記録し、追加のH NNや再loadをしない。

## 細かい二分割と挿入

- 元RGBを保持し、既存OのOtsu支持/インク幅/字高/長さ/境界保護を使う。
  private span divisor **2/4**の二つだけを固定し、両routeが相異なる境界・各2chunk以上を必要とする。
  公開legacy splitterを新支持へ一律置換しない。旧1/2の挙動は保持する。
- 同じ借用Sessionのraw Horizontalへ各cropを渡す。各partは実T一致、EOS停止、
  非空白、scalar数1..T−1、finite confidence0.9..1を必要とする。失敗/LowConfidence/飽和は不採用。
- 両連結全文のbyte一致、Unicode空白を含まない合意、4種類以上のUnicode alphanumericを要求する。
- 現在H結果の非空白列に対し、一つの連続内部挿入で合意全文になる位置が、端も含め全位置中で
  ちょうど一つ必要。先頭/末尾・置換・削除・複数箇所・反復による曖昧さ・挿入境界の既存空白を拒否する。
  Qで検証した純粋挿入helperを再利用し、現在Hの全文字/全空白をbyteで残す。
- confidenceは元Hとdivisor2のscalar加重meanの小さい方。divisor4を重みへ加えず、confidenceを上げない。
  不採用時は元H本文/mean bitsをそのまま保ち、旧sanitizeの位置も変えない。
- 再検査は一度、二routeだけで再帰しない。新partsはraw HorizontalでありH/Sを再起動しない。
  H不採用なら従来Oへ進む。新しいモデルload/再帰pool lockを作らない。

## Test first

停止wrapperを先にRedにし、EOS/LowConfidence/全step完了・空本文・停止後不正logit・形状外tail・
shape/charsetエラーを固定する。128例以上のfinite logits/shapeで従来raw本文/mean bitsとの一致、
既存top-k1停止理由との一致をpropertyで検査する。float strategyのassertion回避に整数から値を作る。

Sの純粋採否はH採用/停止flag/shape metadata、part失敗・EOS/有限値/飽和、両合意、Unicode/空白、
一意挿入と保留時保持をRed→Greenで小さく作る。旧H/O優先と未知shape no-op、RGB不変、
同じSession内の失敗後再利用を実モデルでも検査する。observerは認識結果を変えない。

Oのgeometryを共有化する場合は、既存guardを保ったwrapperから同じ1/2へ渡し、
旧plan/境界/元RGBの一致を既存記録・テストで先に確認する。4の受理拡張はS0の2 Red/Greenと64例PBTを保つ。
実NN前に、新しいdivisor4も含めた全対象の注釈glyph内部切断0と独立crop一致を必要とする。

## 未測定対照と採用gate

Q新72は既知回帰として残し、新しい72を追加NN前に固定する。
新日本語段落の41/45/49、61/65/69、101/105/109 scalar、既知3font・size43/advance52・余白4/12で
54本文を作る。別18図形は3背景×2新図形×45/65/105群とする。
PNG/RGB/GT/glyph bbox/生成条件を固定し、新旧重複を拒否する。既知fontの合成開発対照である。

まず新72×3model×8nativeの前後比較を行い、公式none/width/strictと独立距離を全件照合する。
入力別文字/語の悪化0、新本文のraw文字改善かつ語非悪化が少なくとも一件必要。
無改善/悪化なら全Sを保留し、既知例だけで採用しない。
先行成功後も全既知native414+Q72+S72、全Page/CLI、必要suite/CI・source対応を別に必要とする。
5つの対象外native経路は本文/mean bits不変、Fragment単件/batch/専有batchは各variant内で一致が必要。
公開Page/CLIにも少なくとも一つ生文字改善かつ語非悪化を必要とし、内部APIだけの改善で採用しない。

source/lock/model/charset/入力とfreshな実行物を固定する。画像結果/外側API/load/内部forwardを区別し、
未計測はnullとする。追加NNはSの細かいpartsだけで、H停止metadata自体に追加NNはない。
FVBの別gateと今回を混ぜず、失敗/partial/再試行を保存する。検証完了まで本体へ接続しない。
