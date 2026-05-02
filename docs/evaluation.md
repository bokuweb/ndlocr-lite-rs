# OCR 精度比較（CER/WER）

`ndlocr-lite-rs` と original 実装を同一データで比較するための評価手順です。

## 1. 事前準備

**正解テキストと入力画像**はリポジトリ同梱の `tests/fixtures/eval/` を使います（詳細は `tests/fixtures/eval/README.md`）。

- 入力画像: `tests/fixtures/eval/images/*.png` など
- 正解テキスト: `tests/fixtures/eval/truth/*.txt`（画像と同じ stem）

**予測結果（生成物）**は `tmp/` 配下に出力します（`.gitignore` 対象）。

- Rust 版予測: `tmp/eval/rust/`
- Rust + rule-pack 予測: `tmp/eval/rust_rule_pack/`
- original 版予測: `tmp/eval/original/`

`truth` と各予測ディレクトリでは、**同じ相対パス**で `.txt` を置いてください。

例:

- `tests/fixtures/eval/truth/scaned0.txt`
- `tmp/eval/rust/scaned0.txt`
- `tmp/eval/original/scaned0.txt`

## 2. Rust 版の予測を作る

```bash
mkdir -p tmp/eval/rust

ORT_STRATEGY=system \
ORT_LIB_LOCATION=third_party/onnxruntime-macos-arm64 \
DYLD_LIBRARY_PATH=third_party/onnxruntime-macos-arm64/lib \
cargo run --features onnx -- recognize-page \
  --image tests/fixtures/eval/images/scaned0.png \
  --output-txt tmp/eval/rust/scaned0.txt
```

複数画像を一括生成する場合:

```bash
ORT_STRATEGY=system \
ORT_LIB_LOCATION=third_party/onnxruntime-macos-arm64 \
DYLD_LIBRARY_PATH=third_party/onnxruntime-macos-arm64/lib \
tools/run_eval_rust_predictions.sh \
  --input-dir tests/fixtures/eval/images \
  --output-dir tmp/eval/rust \
  --ort-lib-dir third_party/onnxruntime-macos-arm64/lib
```

行 bbox を広げたい場合は `tools/run_eval_rust_predictions.sh` に `--line-crop-padding N` を追加できます。`run_eval_rust_ab.sh` でも同じオプションを渡せます。

rule-pack を適用した予測を別系統で作る場合:

```bash
ORT_STRATEGY=system \
ORT_LIB_LOCATION=third_party/onnxruntime-macos-arm64 \
DYLD_LIBRARY_PATH=third_party/onnxruntime-macos-arm64/lib \
tools/run_eval_rust_predictions.sh \
  --input-dir tests/fixtures/eval/images \
  --output-dir tmp/eval/rust_rule_pack \
  --rule-pack docs/rule_pack.scaned0.yaml \
  --ort-lib-dir third_party/onnxruntime-macos-arm64/lib
```

## 3. original 版の予測を作る

original 側の実行方法で同一画像の出力テキストを作成し、`tmp/eval/original/` 配下に保存してください。

このリポジトリ同梱の original ソースを使う場合:

```bash
tools/run_eval_original_predictions.sh \
  --input-dir tests/fixtures/eval/images \
  --output-dir tmp/eval/original
```

スクリプトは `ocr.py` の作業ディレクトリが `ndlocr/src` になるため、入力・出力パスを**相対パス**で渡してもリポジトリルート基準に正規化されます。

（注）`ndlocr` 側の依存ライブラリが別途必要です。

## 4. CER/WER を算出

```bash
python tools/eval_ocr_metrics.py \
  --truth-dir tests/fixtures/eval/truth \
  --system rust=tmp/eval/rust \
  --system rust_rule_pack=tmp/eval/rust_rule_pack \
  --system original=tmp/eval/original \
  --normalize basic \
  --require-all \
  --output-csv tmp/eval/report.csv \
  --output-md tmp/eval/report.md
```

標準出力には集計値が出ます（小さいほど良い）。  
`--require-all` を付けると、予測ファイル欠損がある場合に失敗終了します。

ワンコマンドで比較レポートを作る場合:

```bash
tools/run_eval_compare.sh \
  --truth-dir tests/fixtures/eval/truth \
  --rust-pred-dir tmp/eval/rust \
  --original-pred-dir tmp/eval/original \
  --output-dir tmp/eval \
  --require-all
```

Rust 内の A/B（rule-pack あり/なし）だけ比較する場合:

```bash
python tools/eval_ocr_metrics.py \
  --truth-dir tests/fixtures/eval/truth \
  --system rust=tmp/eval/rust \
  --system rust_rule_pack=tmp/eval/rust_rule_pack \
  --baseline-system rust \
  --normalize basic \
  --require-all \
  --output-csv tmp/eval/report_rust_ab_basic.csv \
  --output-md tmp/eval/report_rust_ab_basic.md
```

予測生成〜評価をワンコマンドで回す場合:

```bash
ORT_STRATEGY=system \
ORT_LIB_LOCATION=third_party/onnxruntime-macos-arm64 \
tools/run_eval_rust_ab.sh \
  --input-dir tests/fixtures/eval/images \
  --truth-dir tests/fixtures/eval/truth \
  --output-dir tmp/eval \
  --rule-pack docs/rule_pack.scaned0.yaml \
  --normalize both \
  --ort-lib-dir third_party/onnxruntime-macos-arm64/lib
```

## 5. 正規化モード

- `none`: 正規化なし
- `basic`: 改行統一 + 連続空白圧縮
- `strict`: `basic` + ASCII句読点と空白を除去（文字認識性能を見やすくする）
- `both`: `basic` と `strict` の両方を連続実行（`run_eval_rust_ab.sh` 専用）

## 6. 日本語データでの見方

このツールの WER は空白区切りの token を前提にしています。日本語のように空白なしテキストでは WER が過大・不安定になりやすいため、主指標は CER（特に `--normalize strict`）で比較してください。

## 7. 精度まわりの現状と拡張タスク

### いま済んでいること（ブロッカーなし）

- PARSEQ 行画像の前処理は `ndlocr/src/parseq.py` と整合（全面ストレッチのバイリニア相当、BGR NCHW）。Rust と original の認識差の主要因を解消済み。
- 同梱の定量比較用データは `tests/fixtures/eval/` の **2枚**（`scaned0` と `ndl_logo_ja`）。手順は本書および `docs/examples.md` の「CER/WER 評価」。
- **参考（同一環境・同梱2枚・strict CER の集計）**: Rust 約 **0.034**、original（`ocr.py`）約 **0.043**（2026-03 時点の目安。モデル・ORT・ルールパックで変動する）。
- 文書固有の置換はコアに入れず、`--rule-pack`（例: `docs/rule_pack.scaned0.yaml`）で吸収する方針（`AGENTS.md`）。

### まだ「やるとよい」タスク（優先度は用途次第）

1. **評価データの拡張**  
   `images/` と `truth/` に、レイアウト・紙質・解像度の違う画像と正解テキストを足す。1枚だけでは回帰の網羅性に限界がある。

2. **回帰監視（オプション）**  
   ONNX Runtime とモデルが置ける環境で `tools/run_eval_rust_predictions.sh` と `eval_ocr_metrics.py` を定期実行し、strict CER を記録する。CI に載せる場合は ORT・モデル・（original 比較なら）Python 依存のキャッシュ設計が必要。

3. **Rust と original の差分の切り分け**  
   同一行 bbox の crop を両方から出して比較するデバッグ用ツールは未同梱。必要なら別スクリプトやログ出力で追加する。

4. **閾値テスト**  
   「scaned0 の strict CER が一定以下」などを自動化するには、予測 `.txt` のゴールデンコミットか、CI 上での推論実行が必要（現状の `cargo test` は ONNX なしでも通る設計のまま）。

精度面で「必須の未完了タスク」はありません。上記は、**継続的に品質を上げる・監視する**ための拡張リストです。

## 8. e-Gov 法令検索ページからフィクスチャを追加する

[e-Gov 法令検索](https://laws.e-gov.go.jp/) の URL を指定してスクリーンショットと正解テキスト案を生成する手順は `docs/eval_data_egov.md` を参照してください（`tools/capture_egov_law_eval_fixture.py`）。

## 9. 処理時間と精度を1本にまとめる

`recognize-page` の **wall-clock** と **CER/WER** を同一の Markdown に出すには `docs/evaluation_report.md`（`tools/report_rust_ocr_eval.py`）を参照してください。

**NDLOCR-Lite（`ocr.py`）と並べて比較**する場合は `tools/compare_rust_original_eval.py`（手順は同じく `docs/evaluation_report.md`）。
