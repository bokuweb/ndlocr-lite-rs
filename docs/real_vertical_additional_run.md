# 追加実資料のFVB本体baseline診断（NN前）

先行controls protocolと公開controls-before-nn-v1.zipの6物理画像/11採点表記を変更しない。
本体FVB/e559efbの全5 CI成功を認識開始条件にし、そのsourceとfreshなCLI/Page実行物を固定する。
Cargo/model/charset/PNG/RGB/GTのSHAを認識前後で照合し、未完了時もreceiptと部分結果を残す。

- NN manifestには6物理IDと画像pathだけを渡す。11のGT aliasは採点だけに使う。
  重複/不正ID、未凍結GT、欠けた/未知のphysical参照は認識前に拒否する。
- PARSeq30/50/100のPage eval/default:36 API結果、成功capture3 process。
  CLI DEIM/bands:12 process/12結果。合計48画像結果、264公式CSV行を独立照合する。
  PageRecognizer成功load3/model-session6は同じ固定capture例の範囲とし、CLI内部forward/sessionはnull。
- 既存capture_page_matrix/CLI既定引数を使い、thread設定は従来評価と同じ1。
  GTをモデル選択・補完・閾値へ渡さない。このrunは1 baselineの診断で、改善案の採用比較ではない。
- 入力/GTを観測後に直さない。出力欠落、bbox/行分割、容量付近、空白、字体差等を分類する。
  Page結果は成功した出力本文/矩形/confidenceとして扱い、内部raw/H/O採用traceと混同しない。
- 同一担当の二度目視GTは専門家注釈/学習未重複を意味しない。全aliasを保存して別々に採点する。
  独立して固定したFVC新72、OのCLI trace、FVB既存全件gateへ途中で混ぜない。

## 完了後のモデル不要診断

48出力・264 CSVを保持してから、各採点alias/8経路について生のscalar数、ASCII空白数、
生文字/語距離を保存する。別欄にNFCだけを両側へ適用した診断距離と、実際に変わるscalarの
U+表記・置換先を記録する。NFKC、字体辞書、空白除去はここで使わない。
元文字列/GT/公式CSVを変更せず、NFC診断を製品改善や採用gateの改善数へ数えない。
互換漢字U+FA55（突）の正準分解と、正準同値でない吳/呉・徳/德等を区別する。
