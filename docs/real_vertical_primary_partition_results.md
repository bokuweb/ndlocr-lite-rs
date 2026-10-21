# 実primary二分割診断の結果

6物理列×3モデルの18入力（元36 Page条件）について、既存supported splitterの2経路から
117 partを生成した。全117 raw APIが成功し、実part入力と元座標の4,224,384画素が一致した。
モデルsession loadは3、内部forward数は未計測。全partはEOSで停止した。
model/寸法/RGBで異なる117入力だったため、同一partの反復やdedupはなかった。

| model30列 | 元raw長 | 二経路の結合長 | 結合全文一致 | 元rawを保持して延長できるか |
| --- | ---: | --- | --- | --- |
| 一葉col1 | 23 | 43 / 43 | 不一致（空白位置） | できない |
| 一葉col2 | 22 | 44 / 43 | 不一致（空白数） | できない |
| 一葉col3 | 11 | 43 / 43 | 一致 | 「呉」→「吳」で元prefixが変わる |
| 地獄変col1 | 30 | 31 / 31 | 一致 | 空白追加で元prefixが変わる |
| 地獄変col2 | 30 | 31 / 32 | 不一致（空白） | 一経路だけprefix延長 |
| 地獄変col3 | 30 | 31 / 31 | 一致 | 「土」→「王」で元prefixが変わる |

model30の一致経路組は3、両経路一致かつ完全raw prefixを保持する延長は0。
元の誤字らしく見える箇所を正解に合わせて置換したり、空白を取り除いて合意を成立させたりはしない。
model30 partの平均信頼度は高いが、それだけでは元の文字や空白を変えてよい根拠にならない。
H/O/Q/Sの条件を変更せず、今回の診断から新しい製品policyは採用しない。

全11 GT aliasについてraw/二経路×3モデル×none/width/strictの297公式CSV行を保存し、
全誤り数/分母を独立距離で検算した。これは後処理前の既知6列の診断であり、新規精度改善数は0。
Root変更はtest-only collectorの追加だけで、通常libは442成功/39ignored。
helper未実装compile Red2件→2 pure Green、実装途中のcrop error型変換不足1回も保存した。
Pythonのliteral部分列は未実装Red→Green、packet readerも未実装Red→2 Green。

公開packetは`docs/benchmarks/real-vertical-primary-partition-probe/`。
before 8,672,263 bytes（SHA256 `8b7a3846325c7b8e4cc93babbc247bbdc4f73b22bafbfa368a77826eef47cf19`）、
results 7,510,604 bytes（`4c63d380841b1ee17ce081ff682a9790007004cdcd6edc4386b754b1ab36bbb6`）。
geometryの843 source/fixtureは、既存公開packetへのbyte参照と新規6ファイルで再構成できる。
native実行物/モデルは含めず、fresh compiler artifact・全source・実入力・結果・全before bindingを認証する。
decoder stepの共通検証を抽出し、停止診断の既存4 testsもGreenを保った。
共通化後の追加契約として、途中EOS後の出力継続や低信頼停止条件を超えた文字出力を拒否する。
元の保存結果を変えず、合成した不正traceを先に拒否テストへ加える。
