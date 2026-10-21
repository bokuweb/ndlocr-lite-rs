# Q: 二分割合意による一意な内部欠落の挿入（実装・新対照・追加NN前仕様）

P5診断では、既存Oの羅生門18 Page結果をcfg(test)の観測で再現した。
全本文/矩形/confidence bitsが一致し、18 primary cropと12 O partの実RGBも独立PILと一致した。
実PARSeq session.runは30試行/30成功、Page API18、PageRecognizer3load/model session6loadである。
DEIM内部forwardは未計測null、個別glyph bboxも未注釈であり、glyph欠損0とは主張しない。

30モデルの第1列はraw30 scalar/静的T31で、二つの既存O分割が同じ34 scalarの本文を返した。
元rawには途中の欠落とASCII空白があり、Oは完全prefix/空白列不一致で棄却した。
他16 Page経路組はrawがT−1/Tでないためplanなしだった。これは1資料/1頁の既知診断である。
この結果を一般化したと仮定せず、以下の一案を隔離sourceで固定する。

## 対象と保持

- Hを先に扱い、既存Oが採用したらOを保つ。QはOの同じplan・同じ二分割・同じ実raw部分結果を
  再利用し、O combineが棄却した場合だけ調べる。新しいcrop/NN/loadやT条件の緩和は導入しない。
- Fragment+Auto、既知静的T/Cと実T一致、rawのT−1/T長、元RGB・縦列支持・二つの異なる境界・
  部分文字数<T・各finite confidence0.9..1等は既存Oの条件をそのまま必要とする。
  非対象のraw Session/通常SingleLine/明示Horizontalと、元結果がErrの場合は従来を保つ。
- 二つの連結文字列がbyte完全一致し、4種類以上のUnicode alphanumericを含むことを要求する。
  Qの合意文字列にはUnicode空白が無い場合だけ扱う。空白を含む合意の解釈は広げない。

## 連続欠落の一意性

- 元rawからUnicode空白を除いた列をA、合意文字列をBとする。元raw自体は変更しない。
- BはAより長く、Bの一つの連続部分を削除するとAになることが必要。
  先頭/末尾も含めた全位置を調べ、条件を満たす位置がちょうど一つ、かつその位置がAの内部であることを
  要求する（先頭/末尾は不可）。
  繰返し字等による位置の曖昧さ、複数箇所の挿入、置換、削除は拒否する。
- 対応する元rawの隣接二scalarの間へ、Bで欠けていた文字列だけを挿入する。
  その元境界に空白がある場合は拒否する。境界の既存空白をどちらの語へ所属させるか推測しない。
- 出力は元rawへの文字列挿入だけで構築し、元rawの全文字・全空白を順序どおりbyteで保持する。
  空白を補正/削除したBをそのまま返さない。挿入後の非空白列がBと一致することも検査する。
- confidenceは元rawと主分割のscalar加重meanの小さい方とする。既存confidenceを引き上げず、
  副分割を独立標本として重みへ加えない。元rawのconfidenceもfinite0.9..1が必要。
  いずれかの条件を満たさなければ元raw/mean bitsをそのまま返す。

元の文字/空白を残すため、羅生門第1列にもrawのASCII空白一字が残る予定である。
GTを空白なしへ合わせたり、完全正解へ補正したとは主張しない。法律/契約/書名/字句を条件にしない。

## Test first と隔離検証

純粋helperのRedを先に確認してから最小実装し、Unicode scalar境界、唯一/曖昧/複数の位置、
先頭/末尾、置換/削除、空白境界、部分失敗/不一致/飽和、confidenceのNaN/Inf/範囲を例で固定する。
多数の文字列・空白挿入位置について、採用時に元rawの全byteが挿入前後で保持されることと、
一意位置を総当たりする別oracleとの一致をPBTで検査する。既存O combineを変更しない。

既存Oの414 native RGBと既知全Page/CLIを回帰対照とし、新しい72対照を追加NN前に固定する。
新しい日本語段落を30/31/35、50/51/55、100/101/105 scalarへ切り、既知3書体・size41/advance49・
余白2/14の54本文を作る。別18非文字対照は3背景×2図形×31/51/101群とする。
PNG/RGB/GT/glyph bboxを固定し、新旧の重複を拒否する。既知fontの合成開発対照でありholdoutではない。
既存の本物の空白を含む本文、反復文字、横書き・多列・非文字対照も除外しない。

新72の分割幾何は、元の既知198と合わせた270件（注釈162）で既存のモデル不要記録器へ渡す。
旧legacy/supported・raw/Otsu・3model形状・主/副spanを全て記録し、新旧全chunkのRGBを独立cropと照合する。
supportedの注釈glyph内部切断が一件でもあれば、Qの追加NN前に全案を保留する。
Oの幾何自体は変更しないため、残る既知216の同一sourceによる過去の幾何証跡も保持する。

既存/新規の全対象を公式none/width/strict文字/語と独立距離で照合する。
入力単位の文字/語悪化が一件でもあれば全Qを保留する。新規合成本文の改善を少なくとも一件必要とし、
既知羅生門だけの改善では採用しない。既知実資料の二表記GTも双方を評価する。
source/input/model/charset/lock/実行物SHAと実際のfresh artifact、全呼数/失敗/未計測値を保存する。
通常/native単件・batch・専有session・Page/CLIの対象/非対象の一致と、実PageのRed/Green、必要suite/CIを
揃えるまで本体へ接続しない。FVBの別gateをQの採用根拠へ流用しない。
