# Q新72のnative先行比較（追加NN前仕様）

Qは隔離sourceの実PageでO Red1件/Q Green1件を確認した。既知羅生門1列の本文を回復し、
元ASCII空白・矩形・confidence bits、eval/defaultと不正入力後再利用を保持した。
新規72対照はまだ認識していない。通常lib426成功/38ignored、ONNX nonignored438成功/112ignored。
純粋helper14試験（各512例の2 propertyを含む）とbyte一致するhelperを、既存Oの棄却後だけ接続した。

幾何先行比較は旧198+新72の270入力×6,480条件を計測し、26,506 cropを独立PILとbyte照合した。
supportedのglyph内部切断0、legacy156切断を区別し、既知198の両profile4,752記録は過去公開データと
一致した。split errorは候補なしのまま保ち、成功する入力だけへ対象を狭めない。

## 入力/source/実行物

- 新72の全54本文/18診断図形を全30/50/100モデルで認識する。固定済みliteral RGB/寸法だけを渡す。
  GT/font/PID/本文長の正解はmodel入力やQ採否へ渡さない。
- baselineはGit1f240be/Oと同じ全Rust/Cargo byteに、既存のnative記録器を使用する。
  candidateは実Page Greenと同じQ source。FVBを混ぜない。
  既存O combine/H/通常APIを変更せず、Q helperとO棄却後hookだけを差分とする。
- それぞれONNX lib test artifactのfresh=false、source/lock/model/charset/入力SHAを検査する。
  全source mtimeを更新する場合もbyte不変を確認し、次のCargo実行前に実行物をコピーしSHAを固定する。
  前後のnative実行物SHAは異なることが必要。同じ記録器・同じsource/RGB/モデルを前後で結ぶ。

## 8経路と件数

raw Session Auto/Horizontal、通常Pool Auto/Horizontal、Fragment Pool Auto/Horizontal、
Fragment batch Auto、Fragment専有session batch Autoの8経路を除外せず計測する。
既存記録器の72件modeを用い、全3model/2variantの6コマンドを1 NN processずつ直列に実行する。

- 画像結果は72×8×3×2=3,456。
- 外側APIは各variant/modelで72×6単件+2batch=434、計2,604呼。
- raw/通常/Fragmentの各1loadで、model session load計18。内部forwardは未計測null。
- Fragment単件/batch/専有batchの本文/mean bitsを各variant内で比較する。
  5つの対象外経路はvariant間で全件一致が必要。Qの追加NN/loadは無く、既存O part結果だけを使う。

全終了値、順序、件数、本文/mean bits/errorを保存する。中断/partialを成功として再利用しない。
必要な再試行があれば元結果とattempt数/不明な実呼数を残す。

## 判定

公式none/width/strict文字/語と独立距離で全10,368 CSV行を照合する。
1入力/経路/正規化の文字又は語でも悪化したら全Qを保留する。
新54本文のraw文字改善かつ語非悪化が少なくとも一件必要。既知羅生門だけの改善では採用しない。
無改善/悪化なら反例を保持し、未達gateのまま既知全414やPage/CLIの高価な採用比較へ進めない。
先行比較を通っても採用ではなく、全既知native/Page/CLI、必要suite/CIとsource対応を別途必要とする。
