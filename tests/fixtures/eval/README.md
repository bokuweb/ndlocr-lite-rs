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
| `ndl_accessibility_p2.png` | [国立国会図書館 2025-05-12 報道発表資料](https://www.ndl.go.jp/file/news/fy2025/250512_01/pr250512_01.pdf) 2ページ目本文 | [PDL1.0準拠](https://www.ndl.go.jp/sitepolicy/terms)（ロゴ・ページ番号を除外して加工） |
| `jma_kobe_notice.png` | [気象庁 2026-03-13 神戸市発表区域変更のお知らせ](https://www.jma.go.jp/jma/press/2603/13b/20260313_oshirase.pdf) 本文 | [公共データ利用規約](https://www.jma.go.jp/jma/kishou/info/coment.html)（加工済み） |
| `ndl_logo_ja.png` | [Wikimedia Commons: National Diet Library, Japan text logo (Japanese).png](https://commons.wikimedia.org/wiki/File:National_Diet_Library,_Japan_text_logo_(Japanese).png) | **CC BY 4.0**（国立国会図書館。表示・派生条件に従うこと） |

正解テキスト `truth/ndl_logo_ja.txt` はロゴに含まれる表記に合わせた **1行** です。

## 正解テキストについて

`truth/*.txt` は人手で作成した参照用です。誤りや表記ゆれがある場合は PR で修正してください。

## 画像の追加

新しい画像を追加する場合は `images/` に置き、`truth/` に同名の `.txt` を追加します。

### e-Gov 法令ページからの自動生成（任意）

[e-Gov 法令検索](https://laws.e-gov.go.jp/) の URL から PNG と DOM 由来の正解案を出力する例は `docs/eval_data_egov.md`（`tools/capture_egov_law_eval_fixture.py`）。コミットする場合は **出典 URL・取得日** と、正解を手で直したかどうかをこの README の表などに追記してください。

## 精度評価の拡張（メモ）

- 同梱は **scaned0**、短いロゴ画像 **ndl_logo_ja**、公開お知らせ本文 **jma_kobe_notice**、別様式の公開資料 **ndl_accessibility_p2**（上表参照）。CER/WER の回帰をさらに厚くするには、上記の手順で画像・正解を増やす。
- 手順・指標の説明は `docs/evaluation.md`。Rust と original の比較や rule-pack A/B は `docs/examples.md` の評価セクションも参照。
- 文書固有の誤り修正はコアではなく `--rule-pack` / `--post-dict`（`AGENTS.md`）。

## 神戸市お知らせの追加（2026-10-18）

`jma_kobe_notice.json` に出典・PDF/RGB/真値のhash・描画条件・切り抜き範囲・真値の作成方法を記録。PDF抽出後、画像を目視確認してから初回推論を実施。初回Red後に下端のクリップを修正し、その履歴も同JSONへ保存した。奈良市と同じ様式なので独立様式のholdoutではなく、現在は開発回帰。本文の数字はPDFの幅表記を保持し、none/width/strictで評価する。

## 国立国会図書館報道資料2ページ目（2026-10-18）

`ndl_accessibility_p2.json` に出典・描画・切り抜き・真値作成・hashを記録。ロゴとページ番号を除外し、推論前に抽出真値を全文描画と照合した。気象庁とは異なる書式の初回対照として追加し、2行をまとめたDEIM重複候補の除外前後で既定CLI/帯検出CLIとも本文はbyte同一だった。初回観測後は開発用回帰として扱う。
