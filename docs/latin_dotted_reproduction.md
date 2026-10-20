# 点付きLatin候補の独立再現仕様

## 再現実装前

新候補の証拠はモデル不要で空ディレクトリへ復元する。親packet全体の再採点を毎回繰り返さず、
今回必要な全282,465組のraw/真値/比較前後本文と92,669パスの元RGBを自包する。
比較元Cの公開packet・検証receipt・sourceをSHAで参照し、先行証拠を改変しない。

- ZIPは50MiB未満で分割し、全member SHAと同値aliasをindexする。モデル/実行binaryは含めない。
- 復元先は空を要求する。相対パスの逸脱・archive/member/aliasの不一致を拒否する。
  小さい往復、改変、パス逸脱、同値aliasについてテストを先行する。
- 全RGBと全保存本文を復元し、未変更の公式scorer CLIで153 CSV/153 Markdownを再生成する。
  文字列をstripしない。入力別行と全正規化で文字・単語距離退行0を確認する。
- 新raw59,322件のID/方向/全ROI/確率とf32 meanを照合する。
  実API426組とCLI426件はそれぞれの方向と保存期待値で再監査する。
- 元PNG/JPEG/effective RGB、19,774実NN入力、native描画記録、独立noise再構成の証拠を保つ。
  モデル不要再現と、ローカル実fontを要した1,854回の独立再描画を区別する。
- 凍結owner/policyの数値同一、製品hook以外のNN経路不変を検査する。
  復元sourceを別targetでビルドし、復元RGB全件の実製品出力を再生する。
  元Auto/effective Horizontalの3形状tensor一致も復元後の実Rustで再確認する。

既存の全親評価を再実行したという主張はしない。今回の全153組の採点表を独立に再計算し、
親の証拠は公開archiveと既存の成功receiptのSHAで固定する。
