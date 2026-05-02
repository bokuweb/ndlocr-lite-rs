# e-Gov 法令検索ページから評価フィクスチャを作る

[e-Gov 法令検索](https://laws.e-gov.go.jp/) の法令 URL（例: `https://laws.e-gov.go.jp/law/129AC0000000089`）から、OCR 評価用の **PNG スクリーンショット** と **DOM 由来の正解テキスト案**を生成する手順です。

## 目的

- `tests/fixtures/eval/images/` と `tests/fixtures/eval/truth/` に同名 stem のペアを増やす。
- 以降の CER/WER 手順は `docs/evaluation.md` のまま（`tools/eval_ocr_metrics.py`）。

## 前提

- ページは **JavaScript で描画**されるため、ヘッドレスブラウザ（Playwright）が必要です。
- **正解テキストは DOM の `innerText` 相当**です。ルビ・改行・画面と異なる可能性があるため、必要なら人手で `truth/*.txt` を修正してください。

## 既定のコンテンツ範囲（e-Gov）

セレクタは次の順で **最初に表示可能になった要素**を使います。

1. `#MainProvision` … **現行条文本文**（第一条など。ナビ・旧字体の広いブロックより OCR 評価向き）
2. `article.law` … 法令ブロック全体（目次・表記などを含む）
3. `main.main-content` / `main` … 以降フォールバック

既定の **`--screenshot-target content`** では、**この要素単体の PNG** を保存するため、**画像と正解テキストの範囲が一致**します。  
ビューポート全体を撮りたい場合は `--screenshot-target page`（必要时 `--full-page`）。

## セットアップ（一度だけ）

```bash
python3 -m venv .venv-egov-capture
source .venv-egov-capture/bin/activate   # Windows は .venv-egov-capture\Scripts\activate
pip install -r tools/requirements-egov-capture.txt
python3 -m playwright install chromium
```

## 実行例

リポジトリルートで:

```bash
python3 tools/capture_egov_law_eval_fixture.py \
  --url "https://laws.e-gov.go.jp/law/129AC0000000089" \
  --images-dir tests/fixtures/eval/images \
  --truth-dir tests/fixtures/eval/truth
```

- stem は URL の `/law/<id>` から `law_<id>`（例: `law_129AC0000000089`）。上書きは `--stem my_sample`。
- 別の DOM 範囲を試す場合は `--selector '#...'` を **先頭**に指定（繰り返し可）。記録用に `--metadata path` も併用すると再現しやすいです。
- 旧挙動（ページ全体スクショ + 同上のテキスト範囲と一致しない）は  
  `--screenshot-target page` で切り替え可能です。長いページ全体をページモードで撮る場合は `--full-page` を併用できます。

## 利用上の注意

- 自動取得の頻度・再配布は **e-Gov の利用条件**に従ってください。
- リポジトリに画像・正解をコミットする場合は、`tests/fixtures/eval/README.md` に **出典 URL・取得日** を追記してください。

## ユニットテスト（ネットワーク不要）

```bash
python3 -m unittest discover -s tools -p "test_capture_egov_law_eval_fixture.py"
```
