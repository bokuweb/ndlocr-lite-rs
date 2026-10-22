# R0: H後の内部欠落と二分割合意の診断（追加NN前仕様）

Qは新72×3model×8nativeで本文/mean bitsが全件同じだった。公式/独立10,368 CSV行も一致し、
新規改善0の先行条件未達で全Qを保留した。既知羅生門の1列だけを理由に採用しない。

新54本文×3modelの残誤りをGTで分類すると、現在Fragment結果から一つの連続内部挿入だけで
非空白GTへ一致する例は下記3組だった。raw Session結果はそれぞれ短く、Fragmentは大きく延長している。
既存callgraphからH経由と推論するが、この段階ではcached内部のH採用flag自体を測定していない。

- `q_serif_50_pad14` / model30: 現在43 scalar、GT50。
- `q_bold_50_pad2` / model50: 現在49 scalar、GT50（読点欠落）。
- `q_serif_55_pad14` / model50: 現在54 scalar、GT55（読点欠落）。

これらはGTで選んだ既知の開発診断であり、新規holdoutでも採用比較でもない。
Qの不成立条件を緩めず、製品のH/Oや公開splitterは変更しない。

## 観測のみ

- 270入力の既存モデル不要幾何記録から、対象modelのsupported/Otsu/主副spanの両routeを選ぶ。
  相異なる境界・2chunk以上かを記録し、元RGBの独立cropと全byte一致を確認する。
- NN入力はこの元RGB cropだけとし、GTや復元したい文字列を渡さない。
  同一model/RGB/寸法の重複だけを内容SHAでまとめ、参照元の両routeを保持する。
- 既存のfrozen raw-Horizontal/top-k1記録器を、SHAで固定したOの実行物から実行する。
  30/50の2sessionを直列にloadし、全unique cropの文字列・mean bits・停止/失敗・実入力RGBを保存する。
  追加画像APIとmodel session loadを別計上し、未計測の内部forwardはnullとする。
- 両連結文字列の一致、実T未満の部分scalar数、finite0.9..1 confidence、
  現在Fragmentの全文字/空白を保った一意な内部挿入の可能性を診断する。
  GTは最後の誤り分類だけに使い、文字を合意結果へ足したり採否を入力別に選ばない。

この診断だけではsourceへ新しい回復を接続しない。もし合意で改善が見込めるなら、H採用後にだけ
原結果を保持して検査する別政策・Red/Green・未測定対照・全既知native/Page/CLI gateを先に定める。
合意なし/不支持ならその反例を残す。Qの保留と旧72の無改善を失敗履歴から除かない。
