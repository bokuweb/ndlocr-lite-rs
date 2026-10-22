# R: 非容量付近の短いraw prefixを画像支持と二経路合意で延長する（事前仕様）

FVC v2の全件/Root接続/必須CIを先に完了する。この文書は次候補の固定仕様で、
Root接続前は新対照のNN0準備だけを許し、Rのruntime実装・認識は開始しない。
既存Hのraw長16以下、Oのframe limit付近以外の領域に、別の保守的な条件を検討する。
追加実資料6列は既知診断であり、Rのpositive seedや新規改善へ数えない。

## 固定する候補

- 既存H/Oのどちらも採用しなかったfragment Autoだけを対象とする。
- frame limitをTとしてraw scalar数が17..=T-2。T不明、trailing whitespace、範囲外は追加NN0。
  17は既存Hの上限16の次であり、既存H/Oの条件は変えない。
- 既存Oと同じ寸法/縦長/インク支持/異なる二分割を使う。
  さらにOtsu支持画像の既存fallback glyph anchor数が`max(40, 2 * raw scalar数)`以上であることを要求する。
  これは画線に対し出力が半分以下の欠落候補へ限定するためで、既知列の長さに閾値を合わせない。
- 各partは元RGBを切り出し、同じモデルのraw Horizontalで読む。Otsu画像は認識へ渡さない。
- 既存Oと同じcombination条件: 各partが非空・有限confidence 0.9..=1・T未満、
  二経路の全文がbyte完全一致、全文がraw全文から始まる厳密な延長、異なるalnum4種以上、
  whitespace scalar列を変更しない。元文字の置換/削除/挿入や空白の正規化はしない。
- confidenceの合成は既存Oと同じprimary routeのscalar重み付き平均。棄却/part error時は元rawを完全に保つ。

Rootの形状支持・splitter・既存Oの共通化が必要なら、まず既存Oの全純粋契約を維持するrefactorを行う。
不足・overflow・NaN・空part・不一致・非prefix・空白変化・unsupported時のNN0を先にRed→Greenで確認する。
多様なUnicode prefix/partition境界には例示回帰とproperty testを組み合わせる。

## 新72対照（描画/NN前に固定）

fontは既存Arial Unicode、ヒラギノ角ゴシックW6、ヒラギノ明朝ProNの同じfaceを使用する。
既知fontの開発対照であり、未知font/学習非重複とは主張しない。
各font file SHA、モデルcharset、描画文字の非missing glyphをNN前に確認する。

| cohort | 固定条件 | 枚数 |
| --- | --- | ---: |
| body | 新しい固定段落、長さ67/87/107/127/147/167×3 font×padding4/16 | 36 |
| repeated | 「空」「春夏秋冬」を87 scalarへ反復×3 font×padding4/16 | 12 |
| portrait horizontal | 「港の地図」「No.12」×rows30/44×3 font、row advance64 | 12 |
| nonletter | open_slant_hexagon/offset_crown×groups72/132×white/warm/gray | 12 |

font size44、縦advance52。body段落はgeneratorの固定文字列を途中で選び直さず、
全長が167以上・空白なしであることを先に検証する。GTは切り出した文字列そのまま。
nonletterのGTは描画した非文字図形に対して空、portrait GTは各横行をLFで連結する。
body/repeatedの5,256 glyphを独立描画して全画素/字形bbox/欠落/重複を検証する。
全72 PNG/RGB/GTのSHAとglyph metadataを固定し、既知1,311入力とのPNG/RGB hash重複0を要求する。
重複やglyph欠落はNN前の失敗として保持し、予測結果に応じた入力選別をしない。

## 採否

Source/lock/model/charset/入力/GT/異なるfresh実行物を固定し、既存native collectorで
3モデル×8 API×72入力×前後の3,456画像経路結果を完了する。公式3正規化の10,368 CSV行を独立距離で照合する。
raw比較は1,728組、正規化込み5,184比較。raw/normal/horizontalの非対象5 APIは本文/mean bits完全一致、
非本文36入力は全APIで本文不変、全入力の文字/語悪化0を必要とする。
少なくとも新bodyのfragment Autoにraw文字改善・語非悪化を要求し、同じbodyのbatch/専有batch Autoも単件と一致させる。

8 APIの対象3/非対象5と件数は`docs/vertical_moderate_prefix_api_scope.md`にNN前の訂正を明記した。
既存NN0 packet内の旧仕様と入力/GTは保持し、72対照やthresholdは変更しない。

新規body改善0または悪化ならR全体を保留し、この72へ閾値/条件を合わせ直さない。
成功した場合だけ、同じ凍結sourceの全既知native APIとPage/CLIの全件計画を追加NN前に定義する。
旧API結果やCLIの既知例を新規改善へ流用せず、全件とRoot CIが揃うまで製品へ入れない。
