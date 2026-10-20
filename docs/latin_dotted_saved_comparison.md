# 追加rawの固定候補比較

## 集約実装・採点前の仕様

全59,322 rawは事前固定した条件で取得し、完了済みjobから別々に検査できる。
全モデル・全方向が揃うまでは採用判断を行わない。固定候補の条件を途中結果へ合わせ直さない。

- model/phaseごとのmanifestとrawをIDで対にする。順序に依存せず、両側のID重複・欠落・余分を
  拒否する。各rawは全ROIの1 chunkで、bounds・x_range・本文・空白挿入なしを照合する。
- 元RGBと真値のSHA/寸法を検査する。本文と真値をstripせずUTF-8のまま保存する。
  保存stepのbyte range/character/確率を検査し、以前に実APIとのbit一致を確認した
  emitted_means関数でf32の演算順序を再現する。EOSを平均へ加えない。
- 空白/全角文字/改行の保持、並び替え、重複/欠落/余分、誤ったROI、改変された入力を
  集約のテストで先行する。モデルは30/50/100、phaseはfresh/horizontalだけを許す。
- 比較元の実Rustはf31a87dの製品経路である。現在のtracked Rustとの差がtest-onlyの
  dotted_body module宣言だけで、候補が未接続であることを先に検査する。
  凍結したdefault binaryの既存bridge diagnosticから、実sanitizeのbaselineだけを読む。
  同diagnosticの旧single-pixel候補は新候補の評価に使わない。
- その全baselineを期待値として固定候補diagnosticへ渡し、全件で同じbaselineを再確認する。
  変更されない出力はbaseline、変更される出力は固定fragmentであり、全出力を復元できるようにする。
- 既存223,143組と追加全件をnone/width/strictの文字・単語距離で採点する。
  phase/model別の正式全件CSVに加え、text/noise・engine/palt・font・本文・内側空白・size・
  図形・前後・format・textの紙/外側空白の群も同じ入力別行から集計する。
- 比較前後とも元RGB/NN入力/meanを変更しない。完成したjobだけを改善率へ外挿しない。
  退行があれば全モデル終了後の証拠を保存して採用を止める。

この比較自体のNN forwardは0。新raw取得、後のAPI検証、CLIプロセスは別に数える。
比較先の製品接続後には、全保存rawの実製品出力がこの固定値に一致することを再確認する。
