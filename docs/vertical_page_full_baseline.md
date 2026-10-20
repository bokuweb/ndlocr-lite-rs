# 全918入力の残り270 Page baseline（NN前仕様）

FVBの348×6先行比較を待つ間に、候補に依存しないO基準版のPage baselineを揃える。
FVBを採用する判断ではなく、先行比較が悪化ならFVBは保留する条件を維持する。
FVRの実資料追加検査も別に必要なままである。

既定の918 ID/917 unique RGBは`tmp/vertical-page-supported-output/all-manifest.json`で固定済み。
既にPage測定のあるO444・FVA新96・FVB新108（計648）をIDで除いた270入力を、
全3モデル×eval/defaultで測る。native APIの結果をPageの結果へ流用しない。
この270はnative414とPage444の重複144を除いた入力であり、GT/画像を変更しない。

実行物はOのfresh Page example SHA
6bb4a02663928b5699cb4016b390d97a6074c112d52ef38bc68f220cec252eb5を使う。
そのruntimeは73e9ab4、root1f240beとの差はtest-onlyと確認済み。モデル/charset/lockは
既存固定値を確認し、全入力PNG/GT/source/コマンドSHAを前後で固定する。
GTはrecognizerへ渡さない。3新プロセスを直列に実行し、PageRecognizer load3、model session
load6、外側Page API結果は270×6=1,620を予定する。内部forward/候補内エラーは未計測null。
全失敗を残し、実行が未完了なら成功結果・完全比較とは主張しない。

これはbaselineの拡張だけである。候補比較、公式/独立3正規化、CLI2経路、実Page回帰、
既存suite/CIが揃うまで採用しない。複数のGTや同一RGBのIDを独立資料数と数えない。
