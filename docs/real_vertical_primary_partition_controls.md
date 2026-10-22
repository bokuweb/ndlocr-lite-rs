# 実primary分割診断の認識範囲（追加NN前）

幾何の全18 model/RGB入力で二分割が生成でき、分割不能0、全part数117だった。
今回のraw候補認識は117 partをすべて実行する。同じpartが両経路へ出た場合も実行を省略せず、
重複数を別に記録する。原仕様の「同じpartのみdedup」は異なる入力をまとめない制約であり、
今回はdedupを使わない。同じ実入力の反復結果が不一致なら、その不一致も保持する。

元36 Page条件への対応は維持する。NN manifestはRGBのZIP/member、寸法、モデルとSHAだけとし、
origin/GT/raw期待本文/採用判定はNNへ渡さない。結果は両経路の文字列/空白/信頼度、元rawとの
prefix/部分列関係を報告する。既知6列の診断であり、新規改善のgateではない。
test-only geometryのdefault fullは442成功/39ignored。純粋2 Red→2 Greenに加え、
crop errorからStringへの変換不足によるcompile失敗1回も保持する。
