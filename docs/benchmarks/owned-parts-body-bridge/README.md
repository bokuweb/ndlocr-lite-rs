# 接触した本文主成分と分離部品の支持

比較元はfc5da3a。モデルが正読した `情報」` のcloserを単行処理が落としていたため、
本文主成分の1画素接触と、本文内の大きな分離部品を支持用maskで同時に検査する。
既存の主成分分離・group判定を共有し、元RGB・NN・raw・候補・mean・fragment経路を保持する。
文書種別や単語の置換は追加しない。実装/生成/NN/公開接続の先行仕様はpacketにも保存する。

| 比較 | 条件・モデルの組 | 本文変更 | none完全一致回復 | width完全一致回復 |
|---|---:|---:|---:|---:|
| 旧9群 | 115,023 | 21 | 14 | 19 |
| 新FreeType BASIC対照 | 22,896 | 61 | 37 | 51 |
| 同じ24元画像の明示Horizontal | 72 | 0 | 0 | 0 |
| 合計 | 137,991 | 82 | 51 | 70 |

全群・none/width/strictの文字/単語指標に退行0。82変更のうち74組は少なくとも1指標が改善、
8組は全指標同点。旧2,871画像×3モデルでraw正読後に完全一致を失う組は511→497。
新対照の22画像は旧RGBと重複し、7,610画像は未重複。未重複では50組に適用しnone完全一致30組を回復した。
新対照のnone回復37組には既存RGBの7組が含まれる。画像条件/モデル/方向の出力を独立標本と扱わない。
既知の字形・3font fileの開発対照であり、未知資料や未知フォントでの改善率ではない。

新7,632画像は `情報` / `報情`、3font file、20/32px、本文内空白/端の5closer/外側空白、
上下紙0/1/8、PNG/JPEG75の6,480本文と、裸本文/細線/太線/点の1,152非文字負例である。
負例の出力変更は0。576 pre-JPEG組で本文元画素の保持と図形との重なり0を独立再構成した。
元のseedはPillow12.3.0/FreeType2.14.3のBASIC描画でRGB同一に再現した。
1,116実描画のfont face/index/path/SHA、glyph bitmap SHA、bbox/advance、元画像をNN前に検査した。
この環境にはRAQM/harfbuzzがなく、palt軸は設けず全recordで未指定とした。
Pillowが公開しないnumeric glyph IDやCoreText runを生成したとは主張しない。

CoreTextで別本文を描画した10,944画像、CoreTextの16画像screen、FreeTypeの10画像screenは
固定案の幾何到達性が0だったためNNを実行していない。全入力/真値/描画記録/0適用結果も保持する。
これら10,970画像を精度指標や新NN非悪化標本へ含めない。描画器の違いで1画素接触が変わるため、
原seedの描画方式を先に確認すべきだった経緯とRGB監査も保存する。支持条件はNN結果へ合わせ直していない。

追加認識前に全7,632元Auto/有効Horizontal tensorを3モデル形状でbit照合した。
全ROI raw評価22,968 forward（7,632×3と元Horizontal24×3）が正常終了し、実入力RGB7,656件を照合した。
方向違いを別の独立画像と数えない。支持処理自体に追加NNはなく、API検証の認識実行回数とは分ける。

公開接続は純粋Redと全3モデルSessionのRed（rawは正読、処理後は3モデルとも `情報`）を先行し、
最後の単行fallbackへだけ接続してGreenにした。全137,991保存rawの実製品再生は凍結案と一致した。
公開API2,602組のraw/本文/f32 mean bitとRGB、実単行CLI946件の本文が全件一致した。
通常671件（22 ignored）・ONNX CI対象608件（0 ignored）・example23件が成功、新規Clippy警告0。
未使用importをテストscopeへ移した後も、同じ支持条件の最終sourceを再構築して全検証をやり直した。
修正前の全件成功結果と中断したCLI部分検証も区別して保存する。

狭い丸括弧のH/4幅条件、Latin、複数closer、真縦組みを含むAuto方向の新対照は別施策として残る。
本段階の新規ページNNは0。fragment経路のsource不変性を確認し、前段の287×8ページ実測とは区別する。

## 保存形式と再現

images/pixelsは新FT画像・元RGB・有効RGB・atlas・真値・描画記録、inputsは実全ROI入力RGB、
geometry-images/geometry-pixels/screensはNN未実行の全対照を格納する。
diagnosticsは凍結source/仕様/manifest/候補確率/全比較/テストと失敗も含むログ、textは正式scorer表とCLIを含む。
各zipのSHA/member SHAとaliasは同名jsonに保存する。モデルとOS font fileは同梱しない。
親internal-detached-closerと、その親packetもSHAで固定し、旧全RGBを元の証拠から復元する。

Pillowを利用できるPythonで、次のように復元する（OS fontやモデルは不要）。
多数の小ファイルと比較用出力を作るため、作業用の空き容量は12GiB程度を確保する。
最初の全体復元は容量不足で終了し、前段の検証済み復元コピーだけを整理して再実行した。
元データ・公開packet・固定条件は変更していない。失敗と整理対象の記録も保持する。

```sh
python3 docs/benchmarks/owned-parts-body-bridge/reproduce.py --output /absolute/new/output
```

ネイティブ描画のportable検査は保存trace/元PNG/元RGBのSHA・文字/advance記録の整合性を確認する。
同じfont file・Pillow/FreeTypeを備えた元環境では、保存したverify_native.pyが全1,116描画とglyph maskを実際に再描画して照合した。
portable再現でfont fileなしの再描画を行ったとは主張しない。

## 独立復元の結果

容量を確保した再実行は正常終了し、正式CSV/Markdown306ファイルがbyte一致、3監査も再計算一致した。
153 CSVの全551,781反復指標行を直接確認しても文字/単語退行0だった。これらは正規化/部分群の重複行で、独立標本数ではない。
実入力RGB7,656件、圧縮前の非文字合成1,728組、描画記録3,374件、API2,602組とCLI946件の保存結果/meanも確認した。
同じ凍結Rust binaryで復元後の137,991組を実製品再生し、7,632入力×3形状のtensorもbit一致した。
全archive/member/alias SHAと実行終了記録、最初の容量不足と整理対象は [verification.json](verification.json) に記録する。
