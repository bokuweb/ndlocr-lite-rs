# FVC v2非default設定の固定対照（設定NN前）

新72のdefault比較が完了し、入力別文字/語悪化0と新本文改善が揃った場合だけ測定する。
sourceや閾値をその結果で調整せず、同じ凍結前後CLIを使う。v1の全件成功をv2の証明へ流用しない。

画像は新72から次の8枚をIDで先行固定する（結果やGTによる選別ではない）。
本文arial49/pad6、arial89/pad18、bold69/pad6、serif109/pad18、反復arial「光」/pad6、
横組みserif「No.9」32行、white broken_octagon64、warm offset_kite124。
このsubsetは既知合成設定回帰であり、別の未知資料や新規改善数には数えない。

6 profileを全8画像×DEIM/bands×cascade on/off×前後のCartesianで実行する。
計384 CLI process、公式none/width/strictの1,152 CSV行を独立照合する。

| profile | padding | min confidence | binarize | quality | delta | cascade閾値 | split |
|---|---:|---:|---:|---|---:|---|---|
| padding5 | 5 | 0.3 | 220 | false | 0.03 | 25/45 | true |
| reject_all | 0 | 1.01 | 220 | false | 0.03 | 25/45 | true |
| threshold180 | 0 | 0.3 | 180 | false | 0.03 | 25/45 | true |
| threshold240 | 0 | 0.3 | 240 | false | 0.03 | 25/45 | true |
| quality | 0 | 0.3 | 220 | true | 0.03 | 25/45 | true |
| custom | 13 | 0.0 | 210 | true | 0.01 | 125/200 | false |

split長閾値は全profileで80。profileをdefaultや既存argvへ暗黙に依存させず、全値を明示する。
全argv/画像ID集合をNN前に保存してSHAを結び、重複ID/既知以外/部分/重複commandを拒否する。
profile追加/削除、画像除外、失敗を成功へ数えることはしない。失敗/partialも保持する。
reject_allの64 processは空bodyとstdout LF1個も確認し、他はstdout=body+LFを全件確認する。
全設定の入力別文字/語非悪化を要求し、既知改善で悪化を相殺しない。内部forward/session数は未測定null。

model-free plannerは未実装Redから、全384組の網羅・明示argv・未知/重複入力拒否をGreenにする。
実モデルの設定回帰、feature無効、invalid入力後再利用と全件v2 gate/suiteを区別し、本体接続前に残件を確認する。
