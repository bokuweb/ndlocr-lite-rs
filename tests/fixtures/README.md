# テスト・サンプル用フィクスチャ

## `scaned0.png`（ルート直下）

短いドキュメント例や README のコマンドで参照しやすいよう、`tests/fixtures/scaned0.png` として置いています。

## `eval/`（精度評価セット）

CER/WER などの評価用には **`tests/fixtures/eval/`** を使います。

- `eval/images/` … 評価対象画像
- `eval/truth/` … 正解テキスト（画像と同じ stem の `.txt`）

`eval/images/scaned0.png` はルートの `scaned0.png` と同一内容のコピーです（評価手順では `eval` 側を正とします）。

詳細は `eval/README.md` を参照してください。
