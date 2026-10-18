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
