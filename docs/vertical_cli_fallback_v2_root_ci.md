# FVC v2接続headの必須CI完了

Root 0a0f421c0dd6f564f0ab09ccf2ca46836e29b349 のCI39627691771はattempt2で全5 jobs成功。
モデル不要CI39627691943も成功した。初回のUbuntu/macモデルjobは固定assetのurlopenで
60秒timeoutとなり、NN前に失敗。元logs/attempt1 metadataを保持し、モデル/hash/sourceを
変更せずfailed jobsを再実行した。

再実行した両OSのlogsでknown quality回帰の明示的okとCLI2件のokを確認した。
元upload-artifact ZIP 2点はAPIのsize/digestと一致し、両OS×DEIM off/onの全4出力が
literal120 scalar/UTF-8 360 bytes、stdoutは本文+LF、stderr空、exit0だった。
GT121に対する既知の全角空白欠落は保持する。新規精度改善には数えない。

source列挙helperでもRootの852 fileが受理したv3 freezeと一致した。これでRの隔離実装を
開始できるが、Rの本体接続には別の新72/full gateが必要である。

CI packetの読取器は固定head/run、正しい5 job名と成功、両platformの元artifact digest、
実argv/本文/framing、quality testの実行を検証する。job不足、本文改変、quality行欠損を
成功summaryだけで受理しない。再検証はモデル/NNなしで行う。
