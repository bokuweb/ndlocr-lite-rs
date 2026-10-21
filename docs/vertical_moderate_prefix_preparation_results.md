# R新72対照のNN0準備結果

FVC v2の全件比較中に、Rの推論を開始せず対照とgeometryだけを固定した。
生成helperは3件のmissing-module Redを確認してから実装し3 Green。
packet読取器は2件のmissing-module Red後に実装した。初回読取で停止RGBを非空と
誤って仮定した1 assertion failureも保持し、実際の0 byte停止ファイルを空prefixとして
照合するよう修正した。全5テストがGreen。PNG/GT/描画・分割条件は変更していない。

- 新72画像（body36/repeated12/portrait12/nonletter12）、既知1,311画像とのPNG/RGB hash重複0。
- body/repeatedの5,256 glyphを独立描画して元画像と一致。使用charsetに非空白scalar全数が含まれる。
- 216入力、2,952 part、186,559,200画素を元RGBからの回転・切出しと独立に照合。
- 字形ink bbox途中の分割0。gap/overlap/範囲不足/改変pixel/文字途中の境界を純粋テストで拒否。
- NN0、モデルload0、R runtime未実装。精度改善や採用を示す結果ではない。

初回のENOSPCでは69 PNG、68完全RGB、0 byteの停止RGB1件が残った。
これらを復元可能なgzipと原サイズ/SHA台帳で保持し、出力先とreceipt名だけ変えて再描画した。
読取器でも旧69 PNG/68 RGBの一致と停止RGBのprefix関係を照合する。
font bytesは配布しないため、fontなしの公開再検証はglyphの再描画を主張しない。

公開証跡は `docs/benchmarks/vertical-moderate-prefix/nn0-v1.zip` とindex。
既存のtest-only geometry実行物のfresh build/source証跡は
`real-vertical-primary-partition-probe/before-nn-v1` をcontent hashで参照する。
CIと同じ検証は `python -m unittest discover -s tests -p 'test_moderate_prefix_*.py' -v`。
