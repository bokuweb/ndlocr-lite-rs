# R対照の認識前証跡を独立検証する

新72対照と216 geometry入力は、認識前の凍結データとして公開する。
読取器はPNG/元RGB/GT、固定計画、モデルcharsetの非空白scalar包含、既知1,311入力との
hash非重複を確認する。元fontのbytesは配布せず、既知fontのSHAと描画時の欠落glyph検査を記録する。
font不在の読取器が再描画を実施したとは主張しない。

既存のgeometryビルド証跡とsource bridgeを参照し、source/manifest/実行argvの固定を調べる。
全2,952 partを元RGBの座標から独立に回転・切出ししてbyte比較する。
各経路のrangeは全高を隙間・重複なく覆い、内部境界がいずれのglyph ink bboxも横切らないことを要求する。
不正range・欠損part・改変pixel・glyph途中の境界は、成功summaryが残っていても拒否する。

NN呼出し0、モデルload0、R policy未開始を維持する。これは精度改善の証明ではない。
