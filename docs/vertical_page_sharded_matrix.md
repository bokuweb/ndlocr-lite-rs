# FVB全件比較の時間上限に備えた分割実行仕様

原run39514388558は921入力×全8経路を保持して進める。Page30は前後ともexit0で完了し、
source/fresh artifact/argvと3,684 Page結果が検証でき、公式11,064 CSV行の独立検算も一致した。
残る50/100/CLIは継続中。Page30だけで全件成功としない。

同じfrozen source/model/dataでもrunnerの180分を超える場合に限り、未完了kindを固定の4分割で再測定する。
途中・timeoutの結果は完了として再利用せず、元attemptとログを保存する。

- corpusを元の921物理ID順でround-robinのindex modulo4に分ける（231/230/230/230）。
  同一shardのbaseline/candidateは同じ元RGB・model・全設定を測る。GTで分割や除外を決めない。
- bundle/data/source/asset hashとfreshな実行物の検査を変えない。runnerが使うNN manifestは
  そのshardのID/imageだけとし、source/GTは渡さない。前後とも新たに認識する。
- job metadataにindex/total、全921入力と選択IDを保存する。既存unsharded metadataとの互換を保つ。
  shardの宣言・順序・過不足・重複・一部欠落・異なる分割数・bundle不一致・partialをreaderが拒否する。
- 全件比較時は完了したunsharded Page30と、未完了kindの全4shardsを結合できる。
  同じkindのunshardedとshardedを混ぜない。kindごとに全物理IDをちょうど一回覆う必要がある。
- Page/CLIの元の設定、GT alias、公式none/width/strictと独立距離、入力別文字/語悪化0、
  新規本文生文字改善/語非悪化、CLI本文完全一致を維持する。全44,256 CSV行を必要とする。
  継続/分割の都合で画像や経路を省かず、平均値で個別悪化を相殺しない。
- 外側呼出し・load数は実行形態に合わせて実測metadataを合計する。Page30を一つ、50/100を各4分割なら
  成功PageRecognizer load18/model session load36で、結果総数11,052は不変。
  CLIは3,684プロセス、load/内部forwardは推測せずnull。失敗した元attemptの消費は別記する。

実装前に分割の完全性/順序、1未満/範囲外、重複・欠落shard、unsharded混在をテストでRedにする。
次に最小実装でGreenとし、既存14契約と小さいfake-source/runnerの全件比較で旧/新の同値を確認する。
実NNを増やす前に未完了kindを確定し、完了kindは既存binding付き結果を再利用する。

wrapperはNNを呼ぶ前に、outputの隣の`shard-plan.json`へkind/index/total/全入力数/選択IDと
bundle SHAを保存する。timeoutのSIGTERMでfinallyが実行されなくても、この先行planをartifactへ残す。
成功時は同じ宣言をjob.jsonへも結び付ける。失敗したplanのpathを次の試行で上書きしない。
