# 比例送りの句読点と実空白の追加対照

## 追加認識前仕様

Fの平均advanceによる支持は、同じ送り幅を持つ既存フォント対照で検査した。
句読点の送りだけが詰まる組版では、ASCII空白があっても内部gapが小さくなる可能性がある。
これは未検証の仮説であり、現時点の退行の発見とは扱わない。
Fの条件は変更せず、新しい固定対照で検査する。

- 同じ3フォントファイルのSHAを検証し、macOS CoreTextのfont descriptorから描画する。
  Pillowの同梱runtimeはlibraqmなしのため、OpenType featureを指定できなかった。
  CoreTextの `palt=0` / `palt=1` を明示し、両方のglyph ID・advance・font名・実幅を保存する。
  比例設定が実際に少なくとも句読点のadvanceを変えることを、NNを読む前に確認する。
- 文は `調査結果、提出先` / `利用時間。受付窓口`。
  句読点の直後にASCII空白0/1/2個、3書体、20/32px、2 feature設定、
  横70/100%、上下追加紙0/8px、PNG/JPEG75の全576入力を固定する。
  glyph欠落・別fontへのfallbackを認めない。文字ごとの実fontとglyphを検査する。
- CoreTextの実際の描画を白紙上に描き、黒画素bbox全体＋各辺2pxを保存する。
  空白の有無はtruthに保持する。全体の横resizeはPillow BILINEARで幅floor、高さ不変。
  RGBを確定してから、左右各2px・上下各0/1/2pxの3候補を原画素保持で作る。
- 全2,304 ROI×3モデルを、凍結したwhole ROI exporterで各1forwardとして認識する。
  モデル/charset/source/binary/実入力RGBと寸法、全候補/確率/停止理由を保存する。
- 既存3,446入力のSHAを検証して累積し、Fをそのまま適用する。
  4,022入力×3モデル×3正規化の36,198文字/単語指標を公式scorerで比較する。
  追加結果を見て条件・画像・truthを変更しない。1件でも退行すればFの採用を再検討する。
- 描画器のfixture検査は、空白のadvanceと原画素/候補の寸法・SHA・必要文字・fontの同一性を確認する。
  フォントfeatureが効かない場合は全NN実行を始めず、その制約を記録する。

新しいCoreText描画の合成開発対照であり、未知実資料holdoutとは称さない。
比例送りが実空白を消すという仮説の反証を目的とし、ここで新しいruntime処理は実装しない。
