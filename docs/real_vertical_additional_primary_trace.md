# 追加実資料6列のraw/H/Oを観測する（追加NN前）

FVB本体e559のbaseline36 Page出力で残ったPARSeq30の途中脱落を、GTから原因を推測せず調べる。
FVC全件gateとC1/Q/Sの採否は変更しない。新しい認識方針や辞書置換を加えない。

既知『羅生門』で検証済みのP5/O trace source・fresh build receipt・凍結実行物を再利用する。
そのcollectorは3入力固定のため、事前固定済み追加実資料6列を順序どおり二つの3列batchへ分ける。
NN manifestへはid/imageだけを渡し、元GT・aliases・既存予測は渡さない。6以外や未凍結GTは拒否する。

モデル30/50/100×eval/defaultで新36 Page API結果を観測し、既存FVB本体の36結果と
本文・行数・bbox・f32 confidence bitsを全件比較する。OとFVBのPage fallback差を無視せず、
不一致なら現在の本体の透明なtraceとして扱わない。その場合は不一致を保持し、別にFVB sourceのobserverを準備する。
さらに全primary eventのusing_bands=falseを確認し、O/FVBで異なる未検出fallbackを通っていないことを要求する。
before/traceのPage6経路×11採点表記×3正規化、計396 CSV行を独立距離と照合する。

P5の元Rust/source/lock・実行物・5 assets・6 PNG・NN manifest・driver・既存結果をSHAで固定し、前後照合する。
二つのbatchは成功PageRecognizer load計6、対応するPage model session load計12の範囲を持つ。
実PARSeq forward attempt/successはtrace eventから実測し、通常baselineの内部forward数は推定しない。
failed/partialも保存し、完了batchだけで全36一致を主張しない。既存trace実行物を共有targetから取り違えない。

actual primary cropとO part RGBを元PNGのbbox・回転/切出しと独立照合する。
P5にはH partのRGBや具体的停止理由のeventがなく、Hの採用フラグと前後本文だけが観測される。
Hの詳細は不明として保持し、必要になった場合は別のobserverを仕様・test先行で追加する。
raw復号、Hの採用/停止、Oのplan/棄却、sanitizer前、最終Page本文を分け、
選択outputと実logits T・raw scalar長・停止理由が観測されていない箇所は不明とする。
H/OのeligibilityをPage最終文字数だけから推測しない。字形ごとの欠損は未注釈ならnull。

Q/Sは各新72で改善0となった保留方針のまま。この観測だけで採用を再開しない。
実際の停止と支持が適格であれば、方針を変更しない別の事前固定比較を検討する。
その比較も入力別文字/語悪化0、新しい本文改善、必要な全件gateを別に要求する。
