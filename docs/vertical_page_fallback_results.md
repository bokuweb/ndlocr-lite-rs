# FVAの先行比較結果: 全候補を保留

FVAは本体へ採用しない。全240入力×Page API6経路を、none/width/strictの文字・語で
比較した。公式CSV8,640行は独立bit-vector距離とすべて一致した。本文が変化したのは
47入力/経路組。10入力の19経路組で悪化し、3正規化で57件の棄却条件に該当した。
悪化を伴わず改善した28入力/経路組もあるが、事前の全件非悪化条件を優先する。
入力・モデル・設定・正規化の組を独立な資料数とは数えない。

主な反例は非文字への高信頼幻字だった。white brush40は30/evalで空文字から
confidence0.985090434551239の「AAAA」になり、文字0→4・語0→1へ悪化した。
他の図形には「NNNN」や「the the the the and the the the the」が生じた。
本文にもserif100/model100/defaultの文字97→98（語9→2）という混合の変化が1組あり、
語改善で文字悪化を相殺しない。幾何の字形欠損0や高いconfidenceだけでは採用できなかった。

この比較の有効な新規結果は2,016 Page API結果、既知baselineの再利用は864結果。
容量逼迫時に初回driverがexit120で中断し、childのstderr/終了値を保存できなかった。
中断分の原出力を保持したまま50/100を再実行したため、実際のAPI試行総数は未確定null。
追加で少なくとも430件のtext出力が保存された。corpusのPageRecognizer loadは再試行を
含め8、モデルsession loadは16、内部forward/飲み込まれた候補NNエラーは未計測nullとする。
これらは別途行ったQAの呼出しを含まない。

隔離FVAでは、geometry/recognitionの先行Red→Green、helper10件、通常lib436件、
ONNX nonignored448件が成功した。ONNXのignored113件をこの成功数に含めない。
既存Page回帰は--ignored --exactで実際に1件実行して成功した。
810入力のraw/prepared1,620幾何ケースは796候補を選択し、独立crop/字形検査で欠損0だった。
モデル測定のPage exampleはfresh=falseかつbaselineと異なるSHAで固定した。

raw/manifest/source/失敗/再試行/全CSVは`tmp/vertical-page-fallback-recovery/`へ保持する。
終了済み実行物は復元byte一致を確認したgzipへ、公開済み旧証跡の重複はcanonical archiveへ
参照を記録して整理した。partial出力は削除していない。公開用の再現パケットは後続と合わせる。

本体推論は1f240beのまま。次のFVBは別仕様・別の新規108対照を使い、認識文字数の画像支持と
4種類以上のUnicode alphanumeric文字の支持を検証している。FVAの条件を後から変更した案としては扱わない。
FVBも全件の非悪化と新規本文改善、全918入力×8経路などのgateが揃うまで採用しない。
