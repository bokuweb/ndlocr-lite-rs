# 明示的な横方向白余白の正規化

既定OCR経路には組み込まない実験API。正解文字やdomainルールを使用しない。
`normalize_horizontal_margins(rgb,width,height,margin)` は純白でない全画素を含む最小x区間を取り、
高さと内部空白を保ったまま左右にmargin列を加える。254を含む画素は前景扱い。
結果は `image`, `source_x`, `content_width`, `output_x` を持ち、保持区間の元/先座標を示す。
出力保持区間以外は純白。白紙は入力をそのまま返しsource/output_x=0,content_width=width。
0寸法、RGB長不一致、サイズoverflow、margin>入力幅はエラー。確保失敗もResultで返す。
計算量O(width*height)、出力幅<=3*width。追加依存なし。
単体・propertyテストで白追加不変、非白RGB出典・一意性と境界を先に固定する。
omoの実画像721/725px chunk回帰と28既知development画像の全文A/Bで精度を評価する。
このAPI自体は認識改善や全ページ前景保存を保証しない。

## 拡幅しない明示API

`compact_horizontal_margins(rgb,width,height,margin)` は同じ非白区間と指定白余白で
出力幅が入力幅より**厳密に小さくなる場合だけ**正規化する。それ以外は元の幅・RGB・位置を
完全保持し、source/output_x=0,content_width=widthを返す。白紙も恒等。
不正入力と確保失敗の扱いは既存APIと同じ。既存API/既定pipelineの挙動は変更しない。
目的は外側白の少ない入力に対する拡幅・同幅の位置移動を避けること。
入力へ白列を足す不変性は、元入力が実際に縮小する非白画像の場合にだけ保証する。
縮小しない入力では元canvas維持を優先するため、白追加で適用境界を跨ぐ場合の不変性は保証しない。
単体/PBTで密な全行/同幅非対称/疎な白追加/全画素出典/不正入力を先にRedにする。
omoでは前回と同じ28 development画像で、旧条件を再現して全画像非悪化を検証する。
この条件が通っても独立holdout・ページ全体前景保存・速度/メモリの証明とはしない。
