# Rust 評価レポート（処理時間 + CER/WER）

同じ入力画像集合に対し、**1ページ OCR（`recognize-page`）の wall-clock 時間**と、正解テキストがある場合の **CER/WER** をまとめます。original（`ndlocr`）との比較は、別途予測テキストを用意できれば同一レポートに載せられます。

## 前提

- **ONNX モデル**と **ONNX Runtime**（または `ort` が展開した dylib）が手元で動くこと。
- リリースビルドを使うため、初回は `cargo build --release --features onnx` が走ります（`--no-build` でスキップ可）。

## コマンド例（同梱評価画像）

`tests/fixtures/eval/images/` に `scaned0.png` と `ndl_logo_ja.png` がある前提:

```bash
python3 tools/report_rust_ocr_eval.py \
  --input-dir tests/fixtures/eval/images \
  --truth-dir tests/fixtures/eval/truth \
  --pred-dir tmp/eval/rust_report \
  --output-dir tmp/eval/rust_report_out \
  --normalize strict \
  --ort-lib-dir third_party/onnxruntime-macos-arm64/lib
```

**実測の処理時間・CER/WER はマシン・ONNX・モデル・オプションで変わります。** models と ORT を用意したうえで上記を実行すると、`report.md` に wall-clock 表と精度表が出ます。

`ndlocr/src/config/NDLmoji.yaml` が無い場合は、ndlocr-lite リポジトリの `src/config/NDLmoji.yaml` を取得して `--charset` で渡してください（名前付きパス例: `tmp/ndlocr-lite-src/src/config/NDLmoji.yaml`）。

### 参考（開発者マシン上の一例）

同梱の `tests/fixtures/eval/images`（2枚）、release ビルド、`--normalize strict`、モデル fetch 直後の目安です。**再現保証はありません。**

| 項目 | 値 |
|------|---|
| 合計 wall-clock | 約 21.9 s（2 画像） |
| `ndl_logo_ja.png` | 約 3.4 s |
| `scaned0.png` | 約 18.5 s |
| 集計 CER（2 ファイル） | 約 0.034 |

集計 WER は日本語では参考程度に留め、主に **CER（strict）** を見てください（`docs/evaluation.md`）。

## NDLOCR-Lite（`ocr.py`）との比較

同一の評価画像・正解に対し、**Rust `recognize-page` と NDLOCR-Lite の `ocr.py` をそれぞれ画像単位で計測**し、CER/WER（baseline を original）まで一括するには次を使います。

```bash
# 例: ndlocr-lite を tmp に clone 済み。Python 依存は venv で入れる（README の onnxruntime は環境に合わせて互換版可）。
python3 tools/compare_rust_original_eval.py \
  --input-dir tests/fixtures/eval/images \
  --truth-dir tests/fixtures/eval/truth \
  --output-dir tmp/eval/rust_vs_original \
  --ndlocr-src tmp/ndlocr-lite-src/src \
  --python ./.venv-ndlocr-lite-run/bin/python \
  --charset tmp/ndlocr-lite-src/src/config/NDLmoji.yaml \
  --no-build \
  --require-all
```

一括のみ（従来の `run_eval_original_predictions.sh`）は `--ndlocr-src` で `src` を指定できます。

```bash
tools/run_eval_original_predictions.sh \
  --input-dir tests/fixtures/eval/images \
  --output-dir tmp/eval/original \
  --ndlocr-src tmp/ndlocr-lite-src/src
```

### 参考実測（v1.2.1 モデル・手元・2 枚・strict・2026-05 頃）

**注意**: 画像ごとにサブプロセスで Rust / `ocr.py` を呼ぶため、各画像に ONNX 初期化が乗り、秒数は順序や環境負荷に依存します。`CARGO_TARGET_DIR` を使う環境では、評価ツールはその release binary を参照します。

| 項目 | Rust | NDLOCR-Lite |
|------|------|-------------|
| 合計 wall-clock（2 画像） | 約 8.0 s | 約 9.0 s |
| 集計 CER | 約 **0.034** | 約 **0.043** |
| ΔCER（vs original） | **約 -0.009** | 0 |

上記はあくまで一例です。再現は `tmp/eval/rust_vs_original/report.md` 相当をローカルで生成して確認してください。

予測だけ先に済ませている場合:

```bash
python3 tools/report_rust_ocr_eval.py \
  --no-run \
  --truth-dir tests/fixtures/eval/truth \
  --pred-dir tmp/eval/rust \
  --output-dir tmp/eval/metrics_only \
  --normalize strict
```

original 側の予測ディレクトリがある場合（ファイル stem が truth と一致していること）:

```bash
python3 tools/report_rust_ocr_eval.py \
  --no-run \
  --truth-dir tests/fixtures/eval/truth \
  --pred-dir tmp/eval/rust \
  --original-pred-dir tmp/eval/original \
  --output-dir tmp/eval/compare_out \
  --normalize strict
```

## 出力

`--output-dir` に次が出力されます。

| ファイル | 内容 |
|----------|------|
| `report.md` | 処理時間表 + 精度表（Markdown） |
| `timing_rust.csv` | 画像 stem と所要秒 |
| `metrics.csv` | `eval_ocr_metrics.py` 互換のシステム横断 CSV |
| `metrics.md` | 精度サマリのみ（既存ロジック） |

## ユニットテスト

```bash
python3 -m unittest discover -s tools -p "test_report_rust_ocr_eval.py"
```

## 関連

- 手順の骨格: `docs/evaluation.md`
- original 予測の生成: `tools/run_eval_original_predictions.sh`
