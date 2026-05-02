# 評価用フィクスチャ（CER/WER）

このディレクトリは `tools/eval_ocr_metrics.py` などで使う **入力画像** と **正解テキスト** をまとめます。

## レイアウト

- `images/` … 評価対象画像（ファイル名の stem がキー）
- `truth/` … 正解テキスト（`images` と同じ stem の `.txt`）

例:

- `images/scaned0.png` ↔ `truth/scaned0.txt`
- `images/ndl_logo_ja.png` ↔ `truth/ndl_logo_ja.txt`（短いロゴ文言のサンプル）

### 外部入手画像（ライセンス）

| ファイル | 出典 | ライセンス |
|----------|------|------------|
| `ndl_logo_ja.png` | [Wikimedia Commons: National Diet Library, Japan text logo (Japanese).png](https://commons.wikimedia.org/wiki/File:National_Diet_Library,_Japan_text_logo_(Japanese).png) | **CC BY 4.0**（国立国会図書館。表示・派生条件に従うこと） |

正解テキスト `truth/ndl_logo_ja.txt` はロゴに含まれる表記に合わせた **1行** です。

## 正解テキストについて

`truth/*.txt` は人手で作成した参照用です。誤りや表記ゆれがある場合は PR で修正してください。

## 画像の追加

新しい画像を追加する場合は `images/` に置き、`truth/` に同名の `.txt` を追加します。

### e-Gov 法令ページからの自動生成（任意）

[e-Gov 法令検索](https://laws.e-gov.go.jp/) の URL から PNG と DOM 由来の正解案を出力する例は `docs/eval_data_egov.md`（`tools/capture_egov_law_eval_fixture.py`）。コミットする場合は **出典 URL・取得日** と、正解を手で直したかどうかをこの README の表などに追記してください。

## 精度評価の拡張（メモ）

- 同梱は **scaned0** に加え、短いロゴ画像 **ndl_logo_ja**（上表参照）。CER/WER の回帰をさらに厚くするには、上記の手順で画像・正解を増やす。
- 手順・指標の説明は `docs/evaluation.md`。Rust と original の比較や rule-pack A/B は `docs/examples.md` の評価セクションも参照。
- 文書固有の誤り修正はコアではなく `--rule-pack` / `--post-dict`（`AGENTS.md`）。
