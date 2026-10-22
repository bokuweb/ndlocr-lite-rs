# 短い横書き順序の合成対照

`manifest.json` の全32ラベル×3字体/サイズ=96画像。
選択規則を固定した後、認識前にRGB PNG・truth・SHAを保存した。
逆順対・数字・Latin・全角Latin・漢字・仮名・重複文字を含む。
日本語として不自然な逆順ラベルも、意味による補正を避ける対照として残す。

元フォントはmacOSの角ゴシックW3/明朝ProN。字体ファイル自体は配布しない。
Pillow版、font SHA、サイズと生成済みPNGのSHAはmanifestに記録。
印字bboxに白1pxの余白を足し、幅高は文字列/字体に従う。
生成スクリプトは `../../../docs/benchmarks/short-line-order/generate_synthetic.py`。
再生にはPNGを使うため、フォントのインストールは不要。

これは合成画像の開発回帰であり、未知の実資料への精度保証ではない。
