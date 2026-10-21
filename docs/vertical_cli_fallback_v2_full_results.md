# FVC v2全件gate完了

run [39593424953](https://github.com/bokuweb/ndlocr-lite-rs/actions/runs/39593424953)
（fe0e359371fb7ca7b289092103f6d7f16d41d62f）は全59 jobs成功。
全56 workerの元artifactを保存し、未完了/再試行の除外0で全件を比較した。

| 完了項目 | 件数 |
| --- | ---: |
| 物理入力 / 採点ID | 1,311 / 1,317 |
| 画像経路結果 | 26,220 |
| Page結果 / CLI process | 15,732 / 10,488 |
| 公式CSV行の独立再計算 | 79,020 |
| 正規化込みの採点比較 | 39,510 |
| 入力別の文字/語悪化 | 0 |
| Page本文/bbox/f32 confidence bits変更 | 0 |
| 既知評価範囲のraw gain採点経路組 | 820 |

820は新規820画像ではない。先に固定したv2新72の10本文画像/raw22経路改善を二重計上しない。
PageRecognizer load48、明示的Pageモデルsession load96、内部forwardとCLI session loadは未計測null。

APIの59個のZIP（77,780,477 bytes）をサイズ/SHAと照合した。build stageもnativeを含む
全bytesを確認し、公開packetから4実行物だけを省略した。省略byte/SHAとfresh compilerログを保持し、
公開読取器が省略native blobを再認証できるとは主張しない。

`full-matrix-v1/completed-linux-v1.zip` は15,245,574 bytes、SHA256
`9ec47903ccfbaba1796f5718d87bca034faf6559b17e4a325d242dada43bb78b`。
56元worker ZIP・元比較ZIP・buildログ・native以外のstageを含む。
既存readerへFullBundleV2を渡す回帰をmissing-packet Red→Greenにし、全本文/framing/argv/
Page bits/GT alias/79,020 CSV行/39,510比較をモデル不要で独立再計算した。全gateは明示的true。

ローカルの待機wrapperはENOSPCと同時期にexit120で終了し、元例外は復元不能だった。
watchログと空stdout/stderrの状態を保持した。GHA側の認識は完了しており、再開は元59 artifactの
取得・検証だけで追加NN0。初回の39591981411（build checkout中に取消、NN0）とは別の記録である。

全件と独立再計算の完了後にだけ、事前固定したRoot patchのguardを実行した。
Rootのsrc全92ファイルを凍結v2と照合すると、差分はtest-only geometry登録、private cfg(test)
記録器2個の除去、既存Root32の等価iterator保持だけ。Appと既存finish経路は凍結v2と一致する。
Rootでのfresh CLI/lib実行物・known回帰・feature無効・必須CIは別条件として検証を続ける。
