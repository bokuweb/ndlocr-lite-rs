# FVC全件完了結果の独立再検証

実装前仕様。GitHubのcompare job成功だけに依存せず、完了した全56 workerの原artifactと
build receipt・採点表を保存し、モデル不要readerで再検証する。v1の既知結果から始め、
同じ契約でv2の拡張scopeも検証できるようにする。

- 7種×8分割の完全性、重複/欠落/partial拒否、bundle/build SHA一致を検証する。
- buildのsource一覧を事前bundleへ照合し、own compiler artifactがfresh=falseであることを確認する。
  元build ZIP内の全stage byte/SHAをpack時に検証する。公開packetにはnative実行物/共有ライブラリを含めず、
  build metadata、実compiler stdout、harness sourceを残す。このため公開readerだけでnative実行物の実byteを再認証できるとは主張しない。
- 全workerの実argvを事前計画と比較し、GTを渡していないことを確認する。CLIの本文/stdout末尾LF、
  Pageの全結果/本文を認証し、同じphysical IDの重複を拒否する。
- Pageの全本文/矩形/f32 confidence bitsの前後一致を再検証する。全採点aliasの予測コピーとGTを元bundleへ照合する。
- none/width/strictの全整数指標を独立距離で再計算し、全比較/gain/悪化一覧を再構成する。
  件数はscopeのplanを使い、既知gainを新規画像改善へ重複計上しない。

まずreader未実装Red、実保存データのGreen、欠落/partial/違うbuildの拒否を検証する。
認識は再実行しない。停止attemptやv1の成功をv2完了として流用しない。

v1の完了packetは`docs/benchmarks/vertical-cli-fallback/full-matrix-v1/completed-linux-v1.zip`。
14,297,598 bytes、SHA256 `c9b6b1c2688bc51bf5b45abfa5d779bce16e341e5660e1561fec2deb18672d72`。
全59原artifact（75,874,432 bytes）をprivateに保持し、公開packetには全56 workerとcompareの原ZIP、
buildログ、native以外のstageを収めた。pack時は133,317,941 bytesの全stageを照合し、native4 memberを省略した。
reader未実装Red→全件Green、さらに再認証した欠落/partial/別buildの3変異をいずれも拒否した。
結果は24,780画像出力、74,700独立採点行、37,350比較、悪化0、Page本文/矩形/f32 bits変化0。
raw既知gain798は新規改善へ数えない。v1はfinish省略があるため本体へ採用しない。
