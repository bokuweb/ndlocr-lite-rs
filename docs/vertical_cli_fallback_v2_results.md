# FVC v2の新72先行gate

事前固定した新72画像を、FVB基準と後段保持を修正したv2でDEIM/bands×cascade on/off測定した。
完了した576 CLI process/画像結果、公式none/width/strictの1,728 CSV行の全誤り数/分母が独立距離と一致した。
全288入力/経路組×3正規化で文字/語悪化0。10本文画像のraw22経路組が改善した。
bands on/off各10、DEIM on/off各1で、他の本文と反復12/縦長横組み12/非文字12は全本文が不変だった。
例えばarial49/pad6のbandsは文字誤り50→0（cascade on）、51→0（off）、語誤り22/24→0となった。
既知3 font・同じ固定段落の開発対照であり、未知資料/書体の改善率や22独立画像とは主張しない。

v1で既知になった14画像/30経路組をこの新規改善へ数えない。v2のsourceは初期認識後の既存finishを一度だけ通り、
pure callback2 Red→10 Green、既知quality設定3組、literal CLI2 process、feature無効16 processを別に確認している。
通常lib453成功/40ignored、ONNX非ignored lib465成功/115ignored、ONNX all-targets Clippy成功。
凍結された古いFVB iterator/test警告はroot32efの等価整理で対応済みで、新FVC app/finish testの警告は0だった。

最初の測定は容量不足が起きた間にPython exit120で停止し、129 baseline成功receipt/stderr空を残した。
元例外は復元不能として記録し、0完了gate扱いにした。閉じた自分のRGBの可逆圧縮後、全元bindingを再照合し、
出力先だけ変えて全576を再測定した。partial予測を流用せず、両attemptと実driverを公開packetへ保存した。
内部forward/session数は未計測null。全source/lock/model/charset/入力/GT/実行物は前後同一SHAだった。

`docs/benchmarks/vertical-cli-fallback-v2/new72-cli-matrix-v1.*`へ全結果を保存した。
readerは未実装Red→4 packet tests Greenとなり、全member、source/実argv/body framing、1,728指標行、
864前後比較と全gain/regression一覧をモデルなしで再計算した。停止attemptは全件結果へ数えない。

新72先行gateは成功だが、本体はFVBのまま。非default384条件/1,152 CSV行と、
v2全件1,311物理/1,317採点ID・26,220画像結果/79,020 CSV行・Page全metadata不変、source bridge/必要CIが残る。
非defaultの初回はquality boostのCLI形式を誤り、モデル前のparseで停止した。推論を変えず、
実parser384組と凍結CLIの48設定vectorをNNなしで確認してから、別計画/出力先で再開した。
