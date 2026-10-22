# 縦列容量回復の実モデル回帰

NDL『坊つちやん』（岩波書店、1939年18刷、PID1884801）のPDM表示付きIIIF原JPEGから、
検出矩形そのままの3列をlossless PNGへ保存した。元の見開き、manifest、取得記録、固定本文GT、
原RGB、全出力は精度監査packetへ保存する。ルビ/紙/汚れ/隣のルビ片を画素から除去しない。
同じ書籍の開発回帰で、未知書式のholdoutでもモデル学習との非重複の証明でもない。

- コマ5第2列: `https://dl.ndl.go.jp/api/iiif/1884801/R0000005/full/full/0/default.jpg`
- コマ6第1列: `https://dl.ndl.go.jp/api/iiif/1884801/R0000006/full/full/0/default.jpg`
- コマ3第1列: `https://dl.ndl.go.jp/api/iiif/1884801/R0000003/full/full/0/default.jpg`

warm-zigzag-60は文字列を描かない人工図形。docs/vertical_capacity_controls.mdの先行固定仕様で
描画した18対照の1つ。空の本文を正解と断定せず、元出力を変更しないことを検査する。
`cases.json` のraw/expectedは後処理前のモデル結果と固定selectorの選択結果であり、本文GTではない。
