# Examples

## 1. mock-page を動かす最小例

```bash
mkdir -p /tmp/ndlocr-lite-rs-example
cat > /tmp/ndlocr-lite-rs-example/input.ppm <<'EOF'
P3
2 2
255
255 0 0 0 255 0 0 0 255 255 255 0
EOF

cargo run -- mock-page \
  --image /tmp/ndlocr-lite-rs-example/input.ppm \
  --output-dir /tmp/ndlocr-lite-rs-example/out \
  --line-text "サンプル" \
  --line-count 2 \
  --line-confidence 0.77 \
  --line-orientation vertical
```

生成されるファイル:

- `/tmp/ndlocr-lite-rs-example/out/input.json`
- `/tmp/ndlocr-lite-rs-example/out/input.xml`
- `/tmp/ndlocr-lite-rs-example/out/input.txt`

確認コマンド:

```bash
ls -l /tmp/ndlocr-lite-rs-example/out
```

## 2. onnx 有効の smoke 実行（モデルあり環境）

```bash
cargo run --features onnx -- detect --model /path/to/deim.onnx --image /path/to/image.jpg
cargo run --features onnx -- recognize --model /path/to/parseq.onnx --image /path/to/image.jpg --charset /path/to/NDLmoji.yaml
```

ローカル同梱運用（`models/` 配下に既定名で配置）の場合は、`--model` / `--charset` を省略可能:

```bash
cargo run --features onnx -- detect --image /path/to/image.jpg
cargo run --features onnx -- recognize --image /path/to/image.jpg
```

配置ルールは `models/README.md` を参照してください。
ONNX 生成手順は `docs/model_build.md` を参照してください。

## 3. onnx 有効 smoke テスト雛形を実行

`tests/infer_tests.rs` の `#[ignore]` テストを回す例:

```bash
export NDLOCR_TEST_DEIM_MODEL=/path/to/deim.onnx
export NDLOCR_TEST_PARSEQ_MODEL=/path/to/parseq.onnx
export NDLOCR_TEST_IMAGE=/path/to/image.jpg
export NDLOCR_TEST_CHARSET=/path/to/NDLmoji.yaml

cargo test --features onnx --test infer_tests -- --ignored
```

## 4. PDF を example 側で画像変換して OCR（mock-page）

crate 本体は OCR のみを担当し、PDF→画像変換は example 側で行います。

```bash
cargo run --example pdf_to_image_mock_page -- \
  --pdf tests/fixtures/handwritten_stamp.pdf \
  --output-dir tmp/pdf-example/out \
  --line-text "mock100" \
  --line-count 3 \
  --line-orientation horizontal
```

生成されるファイル:

- `tmp/pdf-example/out/handwritten_stamp.json`
- `tmp/pdf-example/out/handwritten_stamp.xml`
- `tmp/pdf-example/out/handwritten_stamp.txt`

## 5. PDF を実際に文字認識（PARSeq ONNX）

`mock-page` ではなく、`parseq` モデルで実認識を行う example:

```bash
ORT_STRATEGY=system \
ORT_LIB_LOCATION=third_party/onnxruntime-macos-arm64 \
DYLD_LIBRARY_PATH=third_party/onnxruntime-macos-arm64/lib \
cargo run --features onnx --example pdf_to_image_real_ocr -- \
  --pdf tests/fixtures/handwritten_stamp.pdf \
  --output-dir tmp/pdf-real-ocr/out
```

生成されるファイル:

- `tmp/pdf-real-ocr/out/handwritten_stamp.png`
- `tmp/pdf-real-ocr/out/handwritten_stamp.real_ocr.txt`

## 6. 画像を認識して DOCX に変換

`recognize-page` の `--output-docx` を使うと、OCR結果を Word で開ける `.docx` で保存できます。

```bash
ORT_STRATEGY=system \
ORT_LIB_LOCATION=third_party/onnxruntime-macos-arm64 \
DYLD_LIBRARY_PATH=third_party/onnxruntime-macos-arm64/lib \
cargo run --features onnx -- recognize-page \
  --image tests/fixtures/scaned0.png \
  --output-txt tmp/scaned0.txt \
  --output-docx tmp/scaned0.docx
```

辞書補正を併用する場合（YAML）:

```bash
cargo run --features onnx -- recognize-page \
  --image tests/fixtures/scaned0.png \
  --post-dict docs/post_dict.example.yaml \
  --output-txt tmp/scaned0.dict.txt \
  --output-docx tmp/scaned0.dict.docx
```

original 準拠のカスケード推論（30→50→100）を使う場合:

```bash
cargo run --features onnx -- recognize-page \
  --image tests/fixtures/scaned0.png \
  --output-txt tmp/scaned0.cascade.txt
```

`recognize-page` はデフォルトでカスケード有効です。無効化したい場合は `--enable-cascade false` を指定してください。
また、行検出はデフォルトで DEIM ONNX を使います。無効化する場合は `--use-deim-detection=false` を指定してください。
また、長すぎる行に対してはデフォルトで半分分割再認識を行います。無効化する場合は `--split-long-lines=false` を指定してください。
候補再評価（日本語らしさ + 信頼度）を使った quality boost は低速なため、デフォルトでは無効です。有効化は `--quality-boost=true` です。推定が **短い行（PARSEQ-30 向け）**のときは **30 単体**、**中幅行（50 向け）**のときは **50 単体**を、カスケード・100 系の候補に加えて採点し、より自然な日本語になる候補を選びます（`--quality-boost-min-delta` が閾値）。
さらに、条文見出し/箇条書き向けの構造後処理をデフォルトで有効にしています。無効化は `--structure-rules=false` です。

PARSEQ 行画像の前処理は `ndlocr/src/parseq.py` と揃えています（全面ストレッチへのバイリニア相当リサイズ、BGR 順 NCHW）。original Python 実装と認識結果を寄せるうえで重要です。

行 bbox が文字ぎりぎりで欠けるスキャンでは、`--line-crop-padding 1` や `2` で四方向にピクセル分だけ広げてから crop すると改善することがあります（デフォルトは `0`）。評価スクリプトでは `tools/run_eval_rust_predictions.sh --line-crop-padding N` を指定できます。

ドメイン固有の補正は `--rule-pack` で外部YAMLから適用できます:

```bash
cargo run --features onnx -- recognize-page \
  --image tests/fixtures/scaned0.png \
  --rule-pack docs/rule_pack.example.yaml \
  --output-txt tmp/scaned0.rule-pack.txt
```

`scaned0` での残ノイズ抑制に寄せた例（fixture専用）:

```bash
cargo run --features onnx -- recognize-page \
  --image tests/fixtures/scaned0.png \
  --rule-pack docs/rule_pack.scaned0.yaml \
  --output-txt tmp/scaned0.scaned0-pack.txt
```

## 7. CER/WER 評価（正解テキストあり）

評価用の画像・正解は `tests/fixtures/eval/` にあります。手順の詳細は `docs/evaluation.md` を参照してください。

[e-Gov 法令検索](https://laws.e-gov.go.jp/) の URL から評価用の PNG と DOM 由来の正解案を足す例は `docs/eval_data_egov.md`（`tools/capture_egov_law_eval_fixture.py`。既定は `#MainProvision` の **要素スクリーンショット**で画像と正解の範囲を揃えます）。

**処理時間（wall-clock）と CER/WER を1つの Markdown にまとめる**例は `docs/evaluation_report.md`（`tools/report_rust_ocr_eval.py`）。

`tools/run_eval_original_predictions.sh` は `ocr.py` が `ndlocr/src` に移動してから実行するため、入力・出力ディレクトリをリポジトリルートからの相対パスで渡しても正しく解決します。

Rust 内の A/B（baseline と `--rule-pack`）を一括実行する例:

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

`run_eval_rust_ab.sh` も `run_eval_rust_predictions.sh` と同様に `--line-crop-padding N` を付けられます（baseline / rule-pack の両方に同じ値が渡る）。

