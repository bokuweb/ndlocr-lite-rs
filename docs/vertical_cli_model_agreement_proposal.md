# C1: CLIの支持付き単縦列を30/50の完全一致で再確認する（実装・追加対照・NN前）

既知3入力のtraceは、短いdefault結果がinitial100で生じることを示した。
一つの既知bold例だけで30/50が同じ71字を返し、100の24字を完全prefixとして保持した。
他の2例では30/50が一致しないため、一つのモデルやconfidenceだけで切り替えない。
本案は本体FVBを基準に隔離して検証する。進行中FVCやQ/Sと合成しない。

## 不変条件と純粋方針

- CLI cascade onで、初期bucketが100、実pool inputが30=(256,24)/50=(384,24)/100=(768,24)の組だけ。
  任意モデル欠落・単一override・cascade offでは追加NN0。モデルpathだけでshapeを推測しない。
- 元PreparedLineCropは変更しない。FVBの単列/縦横比/幅支持/字形anchor/画線条件を同じRGBへ適用し、
  boundsは支持判定だけに使う。30/50へ渡すRGB/幅/高さは元cropのまま。文字置換や追加sanitizeは行わない。
- anchor数>=6、baseの非空白scalarが1以上ceil(anchor/2)未満の場合だけ再確認する。
  base confidenceはfinite0..1であることを要求する。空出力や既に十分な長さは変更しない。
- まず30を一度だけ呼ぶ。candidateはbase全文をそのままstrict prefixに持ち、より長く、
  空白scalar列がbaseと同一、非空白>=ceil(anchor/2)、alphanumeric4種類以上を要求する。
  confidenceはfinite0.98..1、baseより高く、利用者min confidence以上。NaN minimumは不適格。
  30がエラー/不適格なら50を呼ばずbaseを保つ。
- 次に50を一度だけ呼び、同じ全条件と30との全文byte一致を要求する。両者の小さいconfidenceを採用する。
  エラー/不一致/支持不足なら元baseを保つ。既存の初期cascade、後段quality/context/dict/rule-pack/出力順は保持する。
  Tや容量近傍だけの代理条件、GT、文書種・語句、曖昧字の辞書置換は使わない。

純粋callbackのRed→Green、誤り/非有限値/threshold/完全prefix/空白列/一モデル不一致/呼出し回数と、
採用結果が元baseを保つPBTを先行する。geometry adapterは元crop RGBの不変と不適格時NN0を検査する。
Page/nativeとFVB helper本体の数値は変えず、必要な可視性だけ変更しsource対応を残す。

## 新しい対照を先に固定する

既知font3種/size45/advance53とpadding4/16で、新しい一般文の47/57/67/77/87/97/107/117 scalarを
48本文へ描画する。別12反復本文、12縦長の横書き行群、24非文字図形を加え、新96をNN前に固定する。
非文字は3背景×2図形×32/64/96/128 groups。反復は1/2種類、横書きは28/36 rowsを使う。
PNG/RGB/GT/字形bboxと、既知1,143物理IDに対する非重複を独立確認する。
known fontの合成開発対照であり、未知資料・学習未重複を意味しない。

既知boldのdefault CLIをliteral71字でRed→Greenにし、新96のDEIM/bands×cascade on/offを前後測定する。
768 CLI process/画像結果、2,304公式CSV行を独立検算する。すべての入力・正規化で文字/語悪化0と、
新本文のraw文字改善/語非悪化を必要とする。既知71字だけの改善は数えない。未達なら全C1を保留する。

先行gate後も全既知+新96のCLI4経路、Page6経路の不変、任意モデル/quality boost/min confidenceなどの
設定対照・必要suite/CIを必要とする。FVCを採用する場合は別に合成差分のgateを行う。
source/lock/model/charset/input/GT/fresh実行物/実argvを固定し、failed/partialも保持する。
外側process/pool batchと内部forward/sessionを分け、未計測はnull。本体接続は全gate後とする。

描画前に追加固定: 反復の長さは77、patternは「光」「山川」。横書きrow_textは「午後の窓」「No.7」。
新非文字はnotched_rhombus/paired_bracketsの二種としてgenerator内の点列を凍結する。
新一般文の全文、font SHA、描画設定、generator SHAはgeneration-before-render.jsonへPNG作成前に保存する。
既存FVC rendererの字形配置/独立mask検算を再利用し、C1では本文/対照設定だけを別に固定する。

描画attempt1は横書き4字×28 rowsでheight>=8widthのassertionに達せず停止した（OCR0）。
元source/config/部分画像を保存し、本文の配置は保持して横書き負例だけrow advance64を明示する。
4字row_text・28/36 rowsは保持する。新96の固定完了前のデータ修正で、認識結果は未観測。

純粋段階ではcallback1 Red→1 Green、棄却条件3 Red→4 Green、128例PBT/境界を含む6 Green、
model shape/元RGB adapterの2 Red→8 Greenを確認した。隔離CLI接続後も初期cascade・後段・既存helperはbyte一致。
全default suiteの初回はruntime fixture未同梱で416成功/32 missing-file失敗/38ignoredだったため、
root f99e1feの561既存fixtureをSHA照合して両sourceへ複写した。推論は変えず、再実行448成功/38ignored。
既知71字の実CLI試験は既存matrixと同じdefault引数/DEIM on、全3 PARSeq+DEIM/charset mandatory、
凍結CLIをNDLOCR_TEST_CLI_BINARYで明示し、bodyとstdout LF1個と入力byte不変を確認する。

既知例の実CLIでは基準版の24字本文assertion Red1→候補の71字本文/stdout LF Green1を確認した。
モデル/入力/sourceは前後一致。tempfileの候補raw bodyは保持されず、literal assertionによる検証であることを明記する。
新規改善0として扱い、別runで新96×4経路の前後768 processを開始した。完了まで採用しない。
