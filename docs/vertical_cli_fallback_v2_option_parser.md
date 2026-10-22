# 非default設定argvの実CLI parser回帰（再設定NN前）

最初の384計画はquality_boostを他のboolと同じ`--quality-boost false/true`として生成した。
実CLIは値なしのフラグであり、最初のprocessがexit2でparse停止した。モデルload/NNは0、成功CLI結果も0。
元plan/archive/driver/全stdout/stderrを保持し、推論Rust/モデル/GT/入力は変えない。

Rustの実Cli::try_parse_fromへ凍結384 argvを渡す回帰を追加し、旧argvでRedにする。
profileのbool値は元のまま、trueだけ`--quality-boost`を追加、falseは既存default falseとして省略する。
その他のbool設定は既存の明示値を保持する。v2 planは別archive/別出力先へ固定し、全384がparse成功することを要求する。
元v1の計画は未実行の正しいargvだったと書き換えず、parse不備の失敗記録として保存する。

さらに凍結前後CLIに48種類の設定vectorを、必ず存在しない画像pathで実行する。
parser成功後の画像読込errorまでを確認する。画像読込がモデル構築より前であることと、file error/exit1を要求し、
exit2のparse errorを拒否する。このpreflightもNN/model load0で、精度比較の成功へ数えない。
parser/preflight成功後、同じ8画像/6profileの384実画像processを新出力先へ再実行する。
