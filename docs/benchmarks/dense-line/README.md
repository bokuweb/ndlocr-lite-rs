# 高密度な横長行の短い認識結果を再確認する

実装前仕様: `docs/dense_line_probe.md`、`docs/dense_line_recovery.md`。
比較元は12d3378。79画像は既存20変形＋先行26長短行＋新33対照。
新33画像とtruthは初回推論前に固定。全て既知/合成/機械変形の開発回帰で、
独立した未知資料や79の独立文書を意味しない。

CLIは既定のcascade、DEIM/bandsの2設定。PageRecognizerは30/100それぞれ、
parallelism=1、DEIM threshold=.25、line confidence=.3、drop empty=true。
CLIの高密度再確認は常設100pool、PageRecognizerは供給された同じprimary poolを使う。
単行/生API・既存Pool Auto経路は維持し、追加部分だけHorizontalを明示する。

## 全件の結果

| 経路 | 本文改善 / 79 | 同一 / 79 | 内訳: 既存20 / 長行26 / 新33 |
| --- | ---: | ---: | --- |
| CLI DEIM | 14 | 65 | 0 / 3 / 11 |
| CLI bands | 22 | 57 | 0 / 2 / 20 |
| PageRecognizer30 | 22 | 57 | 7 / 11 / 4 |
| PageRecognizer100 | 7 | 72 | 0 / 3 / 4 |

全79×4×3=948画像別文字距離は非悪化。65本文組はnone/width/strict全てで改善。
残り251本文組はbyte同一。全幅ROIの4和文は同モデル100のchunkで120文字完全一致だが、
crop済みのCLI/runtimeで全4例が完全一致したという主張ではない。

| 例・経路 | 生文字誤り |
| --- | ---: |
| japanese_gothic24 / bands | 117 → 0 / 120字 |
| japanese_mincho24 / bands | 119 → 0 / 120字 |
| japanese_gothic24 / DEIM | 118 → 12 / 120字 |
| japanese_gothic36 / DEIM | 114 → 16 / 120字 |
| japanese_mincho36 / DEIM | 117 → 9 / 120字 |
| scaned0 / PageRecognizer30 | 123 → 87 / 226字 |
| jma_kobe_notice / PageRecognizer30 | 348 → 278 / 1034字 |

**単語指標は一部退行する。** 空白区切りのnone/width WERは6本文組・12指標が悪化した。
100モデルruntimeの総生単語誤りは690→691/997語。`word-regressions.json` と全CSV/MDに残す。
strictの単語指標は非悪化だが、空白を消すstrictだけで結果を選んだわけではない。
文字距離ゲートは先行仕様で決めている。候補内の空白を消す語句補正は追加しない。

## 不採用案と証拠

初回は別領域との重なりを考慮せず、同じ行の2領域を両方長文化した。
2画像×DEIM/runtime100×3正規化で12文字指標が退行した。
`initial-candidate.zip` に全候補本文、runtime領域/信頼度、採点、argv/所要時間、ログを保存。
`initial-candidate.json` はbinary/archive/member SHA。binary自体は配布しない。
zip全504 memberを展開せずSHA検証済み。修正版は、小さい方の面積の50%以上重なる
領域では追加認識を控える。NMSの閾値や検出枠は変えない。

- `evaluation-manifest.json`: 全image/truth/model/charset SHA、比較元binaryと再利用の出所。
- `before/` / `after/` / `truth/`: 全316組の前後本文と固定truth。runtimeの`results.json`はbbox/信頼度付き。
- `before-runs.json` / `after-runs.json`: CLI全argv/終了/所要時間。比較元92本文は前段の同じbinary・image SHAを確認して再利用、66新規CLIは実行。候補158 CLIと前後各158 runtimeページは新規実行。
- `runtime-command.json`: runtimeのモデル/引数/設定/manifest。
- `probe.json`: 全26入力の生100停止理由と3モデルchunk比較。真値は候補選択へ渡さない。
- `audit.json`: 全948文字指標の差分と退行一覧（0）。`totals.json`は各経路の生合計。
- `*-red.log` / `*-green.log`: 未実装の失敗と成功。初回CLI/runtimeの全幅truthを要求したテストは、検出cropによる欠落を考慮して可視部分の復旧とbbox保持へ修正した。

## 再採点

repo rootからPython標準ライブラリのみで実行する。モデルの導入や再認識は不要。

```sh
mkdir -p tmp/dense-line
cp -R docs/benchmarks/dense-line/before docs/benchmarks/dense-line/after docs/benchmarks/dense-line/truth docs/benchmarks/dense-line/evaluation-manifest.json tmp/dense-line/
python3 docs/benchmarks/dense-line/evaluate.py score
```

新規のモデル再実行では、モデルをSHA検証してから現在版のONNX CLIを
`tmp/dense-line/candidate-ndlocr` へコピーし、`evaluate.py after` を使う。
runtime結果は先に `NDLOCR_DENSE_MANIFEST=docs/benchmarks/dense-line/evaluation-manifest.json`
と `NDLOCR_DENSE_RUNTIME_OUTPUT=tmp/dense-line/after` を指定して
`cargo test --features onnx --test dense_page_runtime_probe_tests -- --include-ignored` で保存する。
`NDLOCR_INTRA_THREADS=1` を使用する。feature違いのcargoビルドは同じtargetで並列に走らせない。

追加推論は全chunkの高信頼度を要求し、最初の失敗で元結果へ戻る。
検出欠落、曖昧な重複、36pxで候補信頼度が通らない一部長行、URL/空白残差は残る。
同時実行の所要時間は正式な速度比較ではない。

検証: 通常529件、ONNX対象451件が成功。`onnx-command.json`は全対象/環境を記録する。
libに含めた方針・4実画像・Pool方向互換は既存のONNX CIでも自動選択される。
外部の`dense_line_regression_tests`3件はローカルの全件コマンドに明示追加した。
fmt/diff/対象Clippyは成功（変更箇所以外の既存の本体17警告＋example 1警告）。
