# Rの8 APIと件数をNN前に明示する

既存collector `vertical_uncertain_matrix.rs` の実APIを照合した。
事前仕様の「非対象6 API」は数え誤りで、Fragment Auto単件/batch/専有batchの3経路が対象、
raw Auto/Horizontal・normal Auto/Horizontal・Fragment Horizontalの5経路が非対象である。
全8経路、3モデル、72入力、前後の3,456結果という計画は変更しない。
画像/GT/閾値を変えず、Rのruntime実装/認識前にこの訂正を固定する。

対象3経路は全72で本文/mean bitsが互いに一致することを要求する。
非対象5経路は前後の本文/mean bits完全一致、非本文36入力は全8経路の本文不変。
入力別文字/語悪化0、新bodyのFragment Auto raw改善という採否は維持する。

1 variant・1 modelでは6単件×72+2 batch=434 outer API呼出し、576画像経路結果。
全体はouter API2,604、画像結果3,456、公式CSV10,368、raw比較1,728、正規化比較5,184。
明示的loaderは3 wrapper×3model×2variant=18、内部session forwardは未計測null。
未知/欠損/重複IDや72未満を拒否するplannerを先にテストする。NN0の準備だけであり、
R runtime/認識はFVC v2のRoot必須CI成功後に開始する。
