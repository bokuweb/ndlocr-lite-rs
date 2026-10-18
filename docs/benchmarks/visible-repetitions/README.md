# 画素に裏付けられた反復文字を保持する

実装前仕様: `docs/repeated_glyph_diagnosis.md`、`docs/visible_repetitions.md`、
`docs/contained_line_context.md`。比較元は11b6118。
元127画像に、最初のOCR前に固定した反復160画像を加え、ページ287画像を比較する。
単行はそのうち単行素材と白紙/図形対照の263画像を3モデルで比較する。
既知フォントの合成・変形と既存実資料の開発回帰であり、未知の独立文書holdoutではない。

元の数字4字/反復chunk2回等の後処理は、rawが正読した文字まで削っていた。
8連結成分の数、共通の高さ、非重複の横位置が同じモデル出力の全文字数を支持するときだけ、
反復上限を外す。文字種類の置換・推測による追加・追加推論は行わない。
stateless/Session/Poolの画像を持つ単行APIとページ断片の両方へ適用する。
text-only sanitizer、生出力/候補API、単行と断片の括弧方針は維持する。
画線群の数は文字種類を保証しないため、O→0や1→lなどのモデル誤読は残る。

最初のページ比較は混合英数字1画像・3経路×3正規化の9文字指標が退行した。
DEIMが全体行と細かい断片を重複検出し、全体の反復復旧に伴い二重出力が増えた。
この案を `count-only` として保存した。
その後、同じbaselineに含まれる独立した複数断片・span/gap・認識confidence・文字量が
全体を裏付ける場合にだけ、内側の断片を出力から除くようにした。
認識文字・bbox・f32 confidence・元の順序、全モデル呼出しは維持する。
別行の結合や文書種類/語句に特化した補正は加えていない。

## 最終の全件比較

全7533画像別文字距離は非悪化。今回の比較元11b6118からは単語指標も全7533件で非悪化。
同じ本文の3正規化であり、7533独立資料とは数えない。

| 経路 | 画像数 | 生文字距離改善 | byte同一 | 距離同じで本文変更 |
| --- | ---: | ---: | ---: | ---: |
| CLI DEIM | 287 | 38 | 237 | 12 |
| CLI bands | 287 | 39 | 235 | 13 |
| PageRecognizer30 評価設定 | 287 | 37 | 238 | 12 |
| PageRecognizer30 既定設定 | 287 | 37 | 238 | 12 |
| PageRecognizer100 評価設定 | 287 | 39 | 236 | 12 |
| PageRecognizer100 既定設定 | 287 | 39 | 236 | 12 |
| 単行30 | 263 | 58 | 192 | 13 |
| 単行50 | 263 | 58 | 190 | 15 |
| 単行100 | 263 | 47 | 204 | 12 |

全2511本文組で、生文字距離392組が改善、2006組はbyte同一、113組は本文変更があって距離は同じ。
距離同値の文字種誤読を改善へ数えない。例えばO→0の反復数だけを保持しても、文字種の誤りは残る。
改善のうち371本文組は追加160画像、21本文組は既存素材である。cohort別の全件数も保存する。
先行の未検出行復旧の比較には単語指標12件の退行があり、その記録を取り消すものではない。

通常の全target 568件（最終libを差し替え重複計上せず）、ONNX対象513件、
example 18件、単行フレーミングPython 2件が成功。
別途raw全件再生、API44ページと単行33件の新規一致を確認した。
ONNX全件に含まれるenv指定診断のno-opと、明示envで実行した診断はログを区別して保存する。
fmt/diff/対象Clippy成功。Clippyは既存本体17警告に新しい画素走査のchunks_exactスタイル提案1件、
example既存1警告がある。

## 比較設定

- CLI page: 既定cascade、DEIM / bandsの2経路。
- PageRecognizer30 / 100: parallelism=1、供給primaryだけ、各2設定。
- runtime eval: DEIM=.25、min line=.3、drop empty=true。
- runtime default: DEIM=.3、min line=0、drop empty=true。
- 単行CLI: 30 / 50 / 100の各固定モデル、従来のAuto方向。
- ページ287×6×3=5166、単行263×3×3=2367、計7533画像別文字指標。
- none / width / strictの27表を全件保存し、単語指標も併記する。

単行CLIのstdoutにはprintlnのLFがある。本文真値はLFを持たないため、元stdoutを
`.stdout`として保存し、前後どちらも厳密に1個の終端LFだけを除いて本文を採点する。
全1578出力の`content + LF == stdout`とSHAを検査し、literal truth + LFも別に保存する。
空白全体のstripや原真値の変更はしない。初回のLF混入による誤採点も診断archiveへ保存する。

## rawの診断

184画像・3モデル・原画像/帯cropの1026入力を記録した。
全220前景を含む924入力だけに全文真値を割り当て、部分cropは別扱いとする。
旧後処理は296入力で文字距離を悪化させ、52入力で改善した。
生の正読が崩れた204入力（各モデル65/79/60）は今回の画素支持ですべて保持でき、
その924×3正規化=2772文字指標も非悪化だった。
同じ画像の複数crop/モデルであり、924独立文書とは数えない。

## 実行と再生の区別

比較元CLIは320ページを新規実行、既存254本文は前段11b6118のSHA一致出力を再利用した。
比較元APIは640ページを新規実行、既存508本文とmetadataは前段から再利用した。
比較元単行は789プロセスを新規実行した。CLI/API双方の実行ファイルSHAを記録する。

候補APIはまず全1148ページを新規推論し、その結果へ製品と同じ純粋選択関数を適用した。
選択は全認識後に行うので呼出し履歴は変わらない。変更した全4ページ組と安定対照の
計44ページは新規推論でもbbox/text/f32 confidenceが一致した。
JSONのdecimal再parseはf64の最終桁を変える場合があるため、confidenceは公開APIと同じf32で比較する。
候補CLIは最終ページ処理で全574プロセスを新規実行した。
単行の全789プロセスは反復保持時点で実行済みで、追加したページ選択を通らない。
単行dispatchと認識ソースの一致、全保存byte、最終CLIによる33件の新規byte一致を確認し再利用した。
各段階のsource SHA・binary SHA・再利用一覧を混ぜず記録している。

## 再採点

repo rootでPython標準ライブラリのみ。各archive/memberのSHAを確認する。
元画像やモデルを開かず、保存された本文と真値だけから再採点できる。

```sh
python3 docs/benchmarks/visible-repetitions/unpack_evidence.py --output tmp/visible-repeat-replay
NDLOCR_REPEAT_EVAL_ROOT=tmp/visible-repeat-replay python3 docs/benchmarks/visible-repetitions/evaluate.py score
NDLOCR_REPEAT_EVAL_ROOT=tmp/visible-repeat-replay python3 docs/benchmarks/visible-repetitions/single_cli.py score
```

`--diagnostics`を付けるとraw入力、途中案、source、全テスト/推論ログも取り出す。
新規推論の準備/実行スクリプトと各明示env/argvは診断archiveにある。
再生用のrootで推論も行う場合はmanifestのimage/truthパスと固定モデルを復元する必要がある。
単なる採点に重みの取得や原画像の再生成は不要。

`text-evidence.zip/.json` は本文/真値・APIの全行・CLI stderr/単行stdoutとSHA、
`metrics.zip/.json` は全27組のCSV/MDとSHA、`diagnostic-evidence.zip/.json` は
途中案・raw probe・source snapshots・全Red/Greenと通常/ONNX検証のログを持つ。
圧縮はPRの差分ファイル数を抑えるためで、全memberを検査して展開できる。
`evaluation-manifest.json` は入力/真値/モデル/charset・前後全5022本文のSHAと実行/再生を記録する。

速度/メモリの統制比較はしていないため、高速化は主張しない。
連結した字形、複数成分の1文字、縦向き/Auto回転、欠けた検出枠、1/l/I/O/0等の文字種誤読は残る。
