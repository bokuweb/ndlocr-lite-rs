# FVC v2の新72対照（描画/追加NN前）

後段の保持修正をv1と混同せず、認識前に別の72画像を固定する。
既知font3種・45px・縦cell advance53を保持し、本文長49/69/89/109/129/149×3font×余白6/18の36枚、
反復「光」「風雨」各69字×3font×余白6/18の12枚、縦長横組み「赤い雲」「No.9」×3font×32/42行の12枚、
非文字12枚（背景3×図形2×64/124群、幅63/advance60）とする。
本文はv1と同じ固定段落のprefixで、未知資料や意味的独立標本とは主張しない。
横組みはrow advance64を固定し、縦横比8以上を描画時に検査する。

Pillowの異なるmask取得経路で全縦glyph配置を再構成し、PNG/RGB byte同一を確認する。
FVC全件の既知1,239物理IDに対しPNG/RGB hash重複0を検査し、入力/GT/source/字形annotation/生成前設定を追加NN前に保存する。
失敗した生成は変更/上書きせず保存する。NNで得た本文に基づく描画/真値/閾値調整はしない。

基準FVBとv2のfresh凍結実行物を同じMac/debugで測定する。DEIM/bands×cascade on/off、前後576 CLI process、
全72×4×2×3正規化の1,728公式CSV行を独立検算する。入力別文字/語悪化0と新本文raw改善/語非悪化を必要とする。
v1の既知14本文改善は再度新規改善へ数えず、新72に悪化または改善0ならv2も保留する。

既知の設定保持回帰と新72のdefault gateは区別する。非default品質/padding/min confidence/binarize threshold、
feature無効、エラー後再利用、およびv2 sourceでの全既知gate/必要suite/CIは本体統合前に別に必要。

基準側はv1測定でも使ったFVBのfresh build receipt付き凍結CLIをそのまま再利用する。
対応する282 source files/lock/実行物の同一SHAを認識前後に照合し、共有targetから取り直さない。
v2 CLI/testは修正sourceから新たにbuildして凍結し、基準実行物と異なるSHAを要求する。
既存raw baseline予測を新72の結果へ流用せず、両variantともこの72画像を新規測定する。
