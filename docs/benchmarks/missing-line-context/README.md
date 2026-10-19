# 検出枠が覆わない横書き行の全体文脈を確認する

実装前仕様: `docs/missing_line_context_recovery.md`。比較元270f85a（製品sourceはbae38d3と同一）。
先行の `docs/benchmarks/missing-line-context-probe/` で、一律全体置換にはnative内/高confidenceでも
退行があることと、全文枠を欠く混合A1画像の既定APIに改善候補があることを分離した。
新しい資料/真値は追加せず、固定287画像と単行263画像を全件比較する。
既知資料・合成フォント/変形の開発回帰で、未知の独立文書holdoutではない。

既存の全認識・options・包含断片整理の後、非空結果のbbox和集合に覆われない220前景があり、
横書き帯が1本で文字画素支持を満たす場合だけ、原画像と全文前景を含む帯cropを調べる。
同じ供給primaryのnative比率内の候補だけをHorizontalで追加認識する。
元の非空白文字数で重み付けした平均confidenceを厳密に上回る候補だけを選ぶ。
固定.9のfloor、真値/文書種類/語句の条件、文字の推測置換は使わない。

候補のRGB・text・confidenceを保持し、採用候補bboxの1行として返す。
追加確認失敗時や元枠が全文前景を覆う場合は元結果を保持する。空出力から新たな行は追加しない。
単行/raw APIと辞書/ルールpackを変更せず、CLIでは外部辞書の前に適用する。
CLIの既存cascadeとprimaryのconfidence比較も、通過を仮定せず全574プロセスで評価した。

## 全件結果

ページ287×6×3=5166と単行263×3×3=2367、全7533画像別文字指標と単語指標が非悪化。
単行の全789本文は元出力とbyte同一。2511本文組の生文字距離51組が改善、2450組同一、10組同値変更。
同値の文字種誤読を改善に数えない。3正規化やモデル/cropの違いを独立資料とも数えない。

| 経路 | 画像数 | 生文字距離改善 | byte同一 | 距離同じで本文変更 |
| --- | ---: | ---: | ---: | ---: |
| CLI DEIM | 287 | 11 | 274 | 2 |
| CLI bands | 287 | 8 | 277 | 2 |
| PageRecognizer30 評価設定 | 287 | 9 | 277 | 1 |
| PageRecognizer30 既定設定 | 287 | 10 | 276 | 1 |
| PageRecognizer100 評価設定 | 287 | 6 | 279 | 2 |
| PageRecognizer100 既定設定 | 287 | 7 | 278 | 2 |
| 単行30 | 263 | 0 | 263 | 0 |
| 単行50 | 263 | 0 | 263 | 0 |
| 単行100 | 263 | 0 | 263 | 0 |

改善は既存長短行対照24本文組、追加反復対照27本文組。既存実資料20画像の本文は今回同一だった。
混合A1 JPEGのAPI既定出力は、30モデルが5断片の13誤り→1行4誤り、100モデルが9→4。
100候補のconfidence .893も既存平均 .737を上回るため扱えるが、1/l/I等の文字種誤読は残る。

先行の未検出行復旧には単語指標12件の退行があり、今回の非悪化がその過去の比較を取り消すものではない。
全27組のCSV/MD、cohort別件数と全単語指標を保存する。

## 追加確認と検証

API評価/既定の30モデルは各287ページ中28ページで37回、100モデルは各33ページで46回の追加認識を行った。
4設定合計122ページで166呼出。APIの本文変更は計38組で、bbox/confidenceだけの変更も別に記録する。
別途CLIのdet=.3でも構造補正false/trueを各前後実行し、15→4 / 8→4文字誤り、単語指標も非悪化だった。
診断archive内の `context-calls.json` はf64のdecimal再parseの微差を除くためconfidenceをf32へ戻してmetadataを比較する。
CLIの追加呼出数は計測していない。実行時間/メモリの統制比較もしていないため、高速化は主張しない。

通常全targetは574件（最終libを差し替え重複計上せず）、ONNX対象521件、example18件が成功。
純粋方針の境界/失敗/画素保存と40例の拡大・移動PBT、実モデルの既定回復・評価設定維持・confidence optionを検査した。
最初の合成Cは細すぎて帯検出0本になった。条件を満たす単一帯のfixtureへ直し、元の失敗とPBT seedも残した。
fmt/diff/対象Clippy成功（本体18＋example1の既存警告、新規警告なし）。

APIは全1148ページを新規実行し、test-only observerで全候補のbbox/寸法/平均confidence/結果/失敗を記録した。
モデルを含まない凍結した検証実行ファイルで、元RGBと全before結果に同じ製品選択を再適用し、
全1148ページの呼出順・crop画素・bbox/text/f32 confidenceが保存結果と一致した。
背景準備が画素を変える場合は、両既存準備経路でも選択が同じことを確認している。
候補を真値で選択する処理ではなく、記録済みモデル応答に対する製品方針の全件再生である。

CLIは全574ページを新規実行。その後の変更はtest-only observer/テスト/ドキュメントだけで、
最終sourceを再buildした別binaryでも、本文が変わった全CLI組と安定対照、計41実行がbyte一致した。
単行は3認識sourceとdispatchを照合し、全保存byteと最初の候補binaryによる30新規実行の一致も確認して再利用した。

比較元CLI・候補CLI・最終検証CLI・凍結policy再生binaryのSHAを分けて記録する。
元のAPI推論test実行ファイルは別のtest buildで置き換わる前にSHAを取得できなかったためnullと明示する。
当該driver、製品source、全モデル/input/output/候補応答と、凍結した検証binaryによる全件一致を別の根拠として保存する。
検証binaryのSHAを、元のAPI推論binaryのSHAとして扱わない。

## 証拠と再採点

`text-evidence.zip/.json` は前後全5022本文・真値・API metadata/全候補・CLI stderrと単行stdout、
`metrics.zip/.json` は27組54表、`diagnostic-evidence.zip/.json` はsource/全Red/Green/argv/再利用と検証ログ。
全archive/memberのSHAを検査して展開でき、モデルや元画像を使わず再採点できる。

```sh
python3 docs/benchmarks/missing-line-context/unpack_evidence.py --output tmp/missing-context-text-replay
NDLOCR_REPEAT_EVAL_ROOT=tmp/missing-context-text-replay python3 docs/benchmarks/missing-line-context/evaluate.py score
NDLOCR_REPEAT_EVAL_ROOT=tmp/missing-context-text-replay python3 docs/benchmarks/missing-line-context/single_cli.py score
```

モデル応答のpolicy再生には元入力も必要。repoの固定画像に加え、160画像を以下で復元する。
その後、同じtext展開rootに対して、ONNXなしの明示診断testを使える。

```sh
python3 -m zipfile -e tests/fixtures/repeated_glyph/corpus.zip tmp/repeated-glyph/fixtures
NDLOCR_MISSING_CONTEXT_REPLAY_ROOT=tmp/missing-context-text-replay cargo test --no-default-features --lib replay_missing_context_choices_from_recorded_model_outputs -- --include-ignored
```

今後の検討対象は、実際に印字された閉じ括弧を単行の旧sanitizerが落とす問題、
複数成分を持つ反復字形の未支持、文字種誤読、復旧bboxに含む余白と未知資料での汎化である。
