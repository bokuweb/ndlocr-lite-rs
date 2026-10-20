# 縦列分割合意の公開再現packet

## 組立前の仕様

候補Oのruntime/source・全414RGB・全入力別採否を再現可能にする。K/L/M/N/N2の棄却や保留も残す。
`docs/benchmarks/vertical-partition-agreement/`へ結果を保存する。weights/font/Cargo target/native実行ファイルをGitへ入れない。
生成済み原データと失敗ログはローカルでも削除しない。

- 元414入力のPNG/RGB/GT/glyph bboxとSHAを一つのZIPへまとめる。絶対パスや`..`を展開しない。
  同じ入力を各phaseに複製しない。既知fontの合成対照・実資料・空GTの診断図形の区別を保つ。
- モデル/charset SHA、実raw AutoとHorizontalのliteral token/probability/stops/mean bitsをgzip JSONLで保存する。
  L/M/N/N2の実認識記録だけを集約し、同じmodel/RGB/寸法/方向の再利用を新NNと数えない。
  内部forward未測定はnullとする。raw Horizontalの実入力RGBはSHA/寸法で重複除去しZIPに保存する。
- Oの全1242採否に使うRecognizeResultだけのmanifestと期待結果を小さく保存する。
  mainの実Rust combineを無モデルで再生し、9採用/既知訂正4組拒否/入力別出力・mean bitsを照合できるようにする。
  独立Python検査でtrace/入力bytesを検査し、既存公式scorerでnone/width/strict文字・語を再計算する。
- 最終414×3×8の実API比較、旧baseline342+新72の再利用内訳とsource同一の根拠、H/Page/API/CLI/必要suiteの結果を報告する。
  未完了の検査は未完了と表示し、採用済みと書かない。concurrent測定の時間を速度比較に使わない。
- 実験sourceのoverlay・事前仕様・freeze/commands・失敗ログ・棄却反例を圧縮して残す。
  公開にnative binaryは要らない。ローカルのlossless gzip bindingとSHAは保全し、必要時に復元する。
- 各archive/memberのSHAと件数を検査してから結果docへ数値を書く。再現検査自体も実行してexit0を記録する。

原入力とactual cropの同一性、Rust方針再生、公式精度比較を再実行できることを完了条件とする。
新モデルloadやNN認識を伴わない検査では0と明記する。
