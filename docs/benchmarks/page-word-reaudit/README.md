# 過去のページ単語距離退行と短い語尾の再確認

先行仕様は [`docs/page_word_regression_reaudit.md`](../../page_word_regression_reaudit.md)、比較元は1a66fce。
既存287画像を、PageRecognizer30/100の評価・既定設定とCLI DEIM/bandsの6経路で比較した。
評価設定はDEIM .25・min line .3、既定設定はDEIM .3・min line 0。pool並列数は1、推論threadは1。
入力/真値は元manifestのSHAで固定し、新たな描画真値へ差し替えていない。
既知実資料・合成フォント・機械変形の開発回帰であり、未知の独立実資料holdoutではない。

## 再監査と採用結果

直近の `missing-line-context` の保存本文と、最新基準を新たに推論した全1,722本文組はbyte同一。
`page-band-fallback` で記録した単語距離退行12指標も最新基準に残っていた。

太字画像では「ら、」が短い独立区間になり、生モデルが「5.」と読み、結合空白も入っていた。
既存安全分割の全区間がconfidence .9以上のときだけ、不確かな1〜2文字の末尾と直前区間を
同じ供給モデルに原RGBで再入力する。既存境界・実入力比率・幅・confidenceを確認し、
前半本文が完全一致し、2区間の生本文の文字数が変わらず、末尾に空白を含まない高confidence候補だけを採用する。
前半と末尾を合わせた候補を採り、従来の句読点後の結合空白判定を適用する。

287×6×3正規化=5,166画像別文字指標と単語指標はすべて非悪化。

| 経路 | 生文字距離改善 | 単語距離改善 | byte同一 / 287 |
| --- | ---: | ---: | ---: |
| CLI DEIM | 0 | 0 | 287 |
| CLI bands | 0 | 0 | 287 |
| PageRecognizer30 評価 / 既定（各設定） | 0 | 0 | 287 |
| PageRecognizer100 評価 / 既定（各設定） | 1 | 1 | 286 |

変更は `crop_ja_heavy40_pad6` の100モデル2設定だけ。
「作業が終わった5. 」が「作業が終わったら、」になり、生/width文字距離4→1、
strict文字距離3→1、none/width単語距離2→1。strict単語距離は1のまま。
別箇所の「違いヽ」という挿入誤りが残り、完全一致の回復とは数えない。
他1,720本文組はbyte同一。APIの行数・全矩形と既存全体文脈の診断traceは全組同一で、
confidenceが変わったのもこの2組だけだった。

過去の単語距離退行12指標のうち4指標が当時のbefore水準に戻り、8指標が残る。
当時のbeforeは空出力の単語距離1なので、これも完全一致を意味しない。
残る8指標は丸字体2画像×30モデル2設定×none/widthで、区間内部の生出力に空白が入る。

## 生空白の余白感度で棄却した仮説

この2区間に左右/上下それぞれ0・1・2pxの白余白を加えた18入力を先に固定し、
30/50/100の54回の生candidate exportを記録した。元RGBは全画素同一で、追加画素は純白。
30の両対象は左右2pxで正読するが、100の24px由来区間は左右1pxで正読から誤読へ変わる。
このときconfidenceは約.99854→.99938と上がる。余白の自動追加/信頼度選択は製品へ採用しない。

| モデル | 18条件の生完全一致 |
| --- | ---: |
| 30 | 11 |
| 50 | 18 |
| 100 | 13 |

これは既知2区間の診断。50の結果からページ全体の50モデルを評価済みとはしない。
単純な列投影では複数文字が連続した群になり、空白を一律に消す根拠にはならない。

## 証跡と再採点

`data.zip` と `diagnostics.zip` は、対応するJSONにarchive全体と各memberのSHAを保存する。
入力画像287・真値287、前ページ版/最新基準/候補の全本文、公式CSV/MD、
元fixtureの出典README、実行時source/先行仕様・実バイナリのSHA、model/charset SHA、
APIの矩形/confidence、CLIの全コマンド・stdout/stderr、Red/Green・検証ログと余白診断を含む。
modelと実行バイナリの本体は配布しない。

原manifestは前ページ評価から引き継いだ版を保存しており、入力/真値の固定に利用した。
今回の実行binary/sourceを確認する場合は `current/capture-command.json` と
`candidate/capture-command.json`、各 `cli-command.json` を参照する。
実行時仕様のsnapshotと最終追記を区別している。

```sh
python3 docs/benchmarks/page-word-reaudit/reproduce.py tmp/page-word-reaudit-restored
```

新規ディレクトリへ全memberをハッシュ検証して展開し、同梱した公式scorerで
ページ72表と余白診断6表、2つのページ監査JSONを再生成し、保存版とのbyte一致を要求する。
別ディレクトリで全9,846 memberと78表・2監査のbyte再現を確認した。
モデルの再実行には固定assetsとONNX有効ビルドが必要。実行driver/コマンドは証跡内に保存する。

通常619件、ONNX CI対象523件、example18件、fmt/diff/Clippyが成功。
Clippyの追加警告は0（既存lib18、lib test28・うち18重複）。
APIは前後各1,148回、CLIは前後各574プロセスを新たに実行し、すべて正常終了した。

今回のページ行列のprimaryは30/100。primary50のページ行列、未知実資料への一般化、
追加認識の完全なforward回数と速度は未評価。
