# PageRecognizerの未検出行を同じモデルで復旧する

実装前仕様: `docs/page_band_fallback.md`。比較元はd716fe4。
元103画像に、最初の認識前に固定した24画像を加えた全127画像を比較する。
追加対照は1文字・和文/英文の短文・反復英字/数字・2行文を2字体×PNG/JPEG75で描画したもの。
既知資料と合成・機械変形の開発回帰であり、未知の独立実資料での汎化を示すものではない。

有効なDEIM行がないときだけ、既存の色背景準備と横書き帯検出を使う。
単純な塗り図形を画素で除外し、同じ供給poolのHorizontalで認識する。
実モデル入力比率を超える帯は、原画素を保持する安全な区間の認識に成功した場合だけ追加する。
大きな空白で離れた小語には、必要なら同モデルの全体文脈確認も使う。
追加行は有限かつ0.9〜1のconfidenceを要求してから既存optionsを適用する。
生の検出/単行APIや既存DEIM行のAuto・フィルタは維持する。
CLIにも同じ画素支持を使い、ONNX無効時は元のfeature-disabledエラーを維持する。

## 比較設定

- CLI: 既定cascade、DEIM設定 / bands設定の2経路。
- PageRecognizer30 / 100: parallelism=1、各供給モデルのみ使用。
- runtime eval: DEIM=.25、min line=.3、drop empty=true。
- runtime default: API既定のDEIM=.3、min line=0、drop empty=true。
- 6経路×none/width/strict、全2286画像別文字距離を比較し、単語指標も併記する。
- 比較元の旧103×2 runtime eval本文は前段ZIPのd716fe4本文を全byte再現した。

## 最終の全件比較

全127×6×3=2286の画像別文字距離は非悪化。

| 経路 | 生文字距離改善 / 127 | byte同一 | 文字距離同じで本文変更 |
| --- | ---: | ---: | ---: |
| CLI DEIM | 1 | 126 | 0 |
| CLI bands | 1 | 126 | 0 |
| PageRecognizer30 評価設定 | 22 | 103 | 2 |
| PageRecognizer30 既定設定 | 29 | 96 | 2 |
| PageRecognizer100 評価設定 | 22 | 105 | 0 |
| PageRecognizer100 既定設定 | 29 | 98 | 0 |

762本文組のうち生文字距離104組が改善、654組はbyte同一、4組は本文が変わって文字距離は同じ。最後の4組は30モデルの数字反復2画像×2設定で、`111111`を`一一一一一`と誤読する既知残差であり、改善へ数えない。

空白区切りの単語指標は6本文組（丸字体和文2画像×30モデルの2設定、太字和文1画像×100モデルの2設定）のnone/width、計12指標が1→2へ悪化する。全CSVと退行一覧を保存し、空白削除で隠さない。各経路の総単語誤りは減るが、画像別の全指標非悪化とは主張しない。

## 不採用案

無条件の帯追加は、黒一色を `0.0`、太い縦棒を `CON`、1文字の「雨」を `19` と誤追加した。
画素支持とHorizontalだけの案でも、30モデルが「雨」を `甲冑` / `十四` と誤読する。
認識支持を追加した第三案も文字距離は全2286件非悪化だったが、I反復から高confidenceの英文を
出し、単語誤りが1→12となったため不採用。最終案はnative比率超過帯に安全分割後の確認を要求する。
同じ確認で、全幅認識では途中が壊れたURIが両モデルで全文一致となった。途中案の結果は採用結果へ混ぜない。

離した1文字の単独認識は低confidenceだったが、巨大な白帯だけを省いた全体画像は
両モデルが0.999以上で正読した。上下余白/Autoの変更では安定せず棄却した。
I反復をnative比率まで白余白で広げる案や周期的複製も、別単語を高confidenceで出す等のため棄却した。
これらの実測・生出力・失敗テストを診断archiveへ残す。100モデルのI→一や反復数の欠落は
モデル/既存sanitizerにも残り、真値を変更せず全件の誤りへ計上する。

## 再採点

repo rootでPython標準ライブラリのみ。全archive/member SHAを検証して戻す。

```sh
python3 docs/benchmarks/page-band-fallback/unpack_text_evidence.py --output tmp/page-band-replay
NDLOCR_BAND_EVAL_ROOT=tmp/page-band-replay python3 docs/benchmarks/page-band-fallback/evaluate.py score
```

新しいruntimeの推論は `NDLOCR_BAND_MANIFEST=docs/benchmarks/page-band-fallback/evaluation-manifest.json`、
`NDLOCR_BAND_RUNTIME_OUTPUT=tmp/page-band-fallback/after`、`NDLOCR_INTRA_THREADS=1` を指定し、
`cargo test --features onnx --test page_band_runtime_probe_tests record_page_band_runtime_matrix -- --include-ignored`
を明示選択する。同targetの他のignored testは診断用である。
CLIはONNX binaryを `tmp/page-band-fallback/candidate-ndlocr` へコピーし `evaluate.py after` で全254プロセスを実行する。
再推論前にはunpackで比較元とmanifestを既定rootへ復元する。

追加推論を含む速度・メモリの統制比較は未実施で、高速化は主張しない。
検出枠の欠落や曖昧な重複、句読点/URL/空白残差、低信頼度の正読の棄却と高信頼度の誤読は未解決。

## 検証記録

仕様→Red→Greenを小さい挙動ごとに実施。局所の例示/PBTに加え、実モデルの未検出長行、
短文字、疎な語、図形、pool方向指定、load/from_pools、ページ順、option、失敗後の再利用を検査した。
CLI側実装の全254プロセスは再ビルドしたbinaryで実行した。その後の修正は別のPageRecognizer APIと
privateな帯復旧moduleだけで、CLIのapp/cached/pipelineのSHAが同一であることを確認して結果を保持した。
最終API executableのSHAと、CLI予測を生成したbinaryのSHA・ソースは個別に記録する。

`text-evidence.zip` / `.json` は全前後本文・truth・runtime枠/信頼度・CLI stderrを収録し、
archive/memberごとのSHAを検証する。`evaluation-manifest.json` は画像/真値/モデル/charset、
CLI binary・候補runtime binary・実装ソースと全1524予測本文のSHAを記録する。
比較元runtimeの実行ファイルSHAは取得していなかったため、nullとして明示し、source d716fe4と
全行metadata・旧103画像の206本文のbyte再現を根拠として区別している。
`diagnostic-evidence.zip` / `.json` は不採用3案の全出力/採点、原入力probe、ソースsnapshot、
全Red/Green・通常/ONNX/Clippyログを保存する。途中のfixture拡張子の誤記による失敗も記録した。
最終テストではPNG/JPEG両方を正しいファイル名で実行している。

最終検証は通常553件・ONNX対象489件が成功。fmt/diff/対象Clippyも成功（既存の本体17警告＋example 1警告）。
