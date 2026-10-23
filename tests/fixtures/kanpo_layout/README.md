# 官報の横書き段組み評価

- `first.png`, `numbered.png`: lawrenceanumの既知の告知crop。調整用355文字。
- 日付IDの3 PNG: 引き継ぎ時の既知全ページ。全文正解はなく、文字数をrecallにしない。
- `heldout-page.png`: `20261019g002290021` の原PDF内画像。3509×2480px。
  既知5画像と異なるページを、主担当agentが画像を拡大して直接転記した。
  OCR結果を転記元にしていない。4段全体2,613文字（NFKC+空白除去）。
  `heldout-col1.txt`〜`heldout-col4.txt` は物理段ごと、`heldout-page.txt` は左→右の結合。
  第1264号の本文は第3段末から第4段頭へ続く。
- `frozen-before-recognition.json`: 正解/元PNGを認識前にSHA256で固定した記録。
  GTの表記/行/空白は採点条件と別で保持し、誤認識に合わせて修正しない。
- `manifest.json`: 両エンジン共通の画像・SHA・正解・寸法。生出力に正規化をかけない。

原PDF SHA256: `46b72dfec392cf7a914784d8e6bea3fcc08a404992f8f639ca1f5db5b5039741`。
抽出はlawrenceanumの既存image-source抽出物をコピー。ページ画像のSHAはmanifestを参照。
画像を見た一人（agent）の転記であり、人間二名の独立校正済みcorpusではない。
同じ組版の1ページでの評価を一般文書・別レイアウトへ外挿しない。
官報の公示資料で、モデル学習や人名辞書の作成には利用していない。

## 最終の独立確認と回帰対照

途中の`heldout-page`と`confirmation-*`は失敗の切り分けにも使ったため開発用へ移した。
最終確認は、認識処理と既定閾値0.25を固定した後に別ページ
`20260727g001670057`から転記した`final-holdout-*`（1,069文字、隣接する完全な2段）。
`final-holdout-frozen-before-recognition.json`に元PNG・ROI・全GTのSHAを保存し、
`docs/benchmarks/kanpo-layout/source-before-final-holdout.json`で実装を先に固定した。
OCR結果を見てからGTを調整しない。

先に開いた`20260722h017520032`は2480×3509の縦書きで対象の横書き段組みと異なった。
これを`vertical-control.png`として残し、旧版/採用版の本文byte一致を検証する。
配置だけで対象を選び、精度を見て不利な画像を除外したものではない。
