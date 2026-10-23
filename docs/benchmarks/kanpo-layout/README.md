# 官報横書き段組みの比較記録（2026-10-23）

採用版は `adaptive-source.zip`。基準commitは `0ba1c0007c12c91f2527d40a9033ca7815d18795`。
基準commitの別checkoutへ展開する差分付きソースsnapshotで、既存の未変更fixture/supportは
基準checkoutを使う。モデルは含めず、各runのinputs.jsonにSHA256を保存する。
`adaptive-identities.json` は採用binary/モデル設定、`adaptive-source-manifest.json` は全同梱sourceのSHA。

`raw-evidence.zip` は全試行の本文、stdout/stderr、argv、モデル/binary hash、Tesseract raw JSON、
recognize-pageの段階診断を保存する。ZIP内の各fileは`raw-evidence-manifest.json`で照合可能。
展開後は`tools/summarize_kanpo_layout.py`で生出力のhashと公式Levenshteinを再計算できる。

- `baseline-run`: 元binary・検出0.25。精度の基準。最初のcropはbuildと重なったため速度基準にしない。
- `columns-run`: 段分離初版・検出0.9。1文字行が全ページの段順を無効にする失敗も含む。
- `final-run`: 段順修正後・検出0.9。途中版の歴史的なdirectory名であり採用版ではない。
- `confirmation-run`: 同じ途中版の別ページ2段。0.9でもDEIM14行が残る大量欠落を保持する。
- `adaptive-run`: 採用版・検出0.25。元6画像、各engine3反復。
- `adaptive-confirmation-run`: 採用版で前記失敗2段を再測定。ここは開発用へ移した。
- `independent-adaptive-run`: sourceとGTを認識前に固定した最終未使用2段、各engine3反復。
- `before`, `after-traces`: 実CLIの各段階の診断。診断付き実行時間は速度集計へ混ぜない。
- `existing-eval`: 同梱5画像の元版/採用版。none/strictを公式実装で採点。全5本文byte一致。
- `vertical-control`: 横書き独立評価へ選ぶ前に見た縦書き1画像を、元版/採用版の回帰対照として残す。

独立確認の最終sourceは`source-before-final-holdout.json`、GT/ROI/sourceは
`tests/fixtures/kanpo_layout/final-holdout-frozen-before-recognition.json`で固定した。
最初の追加全文ページと最初の追加2段は不具合確認にも使ったので、最終の独立性能とは区別する。
辞書・正解語置換・GT書換え・ページ名による条件分岐を使用しない。

## 再実行（repository root）

```sh
cargo build --release --features onnx --locked
python3 /Users/bokuweb/ghq/github.com/bokuweb/lawrenceanum/tools/kanpo-ocr/benchmark.py \
  --manifest tests/fixtures/kanpo_layout/final-holdout-manifest.json \
  --ndlocr-bin target/release/ndlocr-lite-rs --ndlocr-models models \
  --tesseract-model /Users/bokuweb/ghq/github.com/bokuweb/lawrenceanum/.cache/kanpo-227/tesseractjs-ocr62-frozen2/runtime/models/jpn.traineddata \
  --det-conf-threshold 0.25 --min-line-confidence 0.3 \
  --output tmp/kanpo-independent-new --repeats 3
python3 tools/summarize_kanpo_layout.py \
  --manifest tests/fixtures/kanpo_layout/final-holdout-manifest.json \
  --run independent=tmp/kanpo-independent-new --output tmp/kanpo-independent-score.json
```

出力先は毎回新規directoryを指定する。元6画像は`manifest.json`、開発へ移した2段は
`confirmation-manifest.json`に差し替えて再実行できる。

診断は通常のTXTとは別に保存する（この例は既定閾値0.25、構造ルール無効）。

```sh
target/release/ndlocr-lite-rs recognize-page \
  --image tests/fixtures/kanpo_layout/final-holdout-two-columns.png \
  --charset models/NDLmoji.yaml --structure-rules=false \
  --output-txt tmp/kanpo.txt --trace-json tmp/kanpo.trace.json
```

検査は`cargo test --locked`、`cargo test --locked --features onnx --lib`と、
`--test long_line_cli_regression_tests --test vertical_fallback_cli_tests --test page_band_cli_tests --test cli_trace_tests -- --ignored --test-threads=1`を使用。
Python採点/固定GTの検査は`python3 -m unittest discover -s tests -p test_kanpo_layout_report.py`。
