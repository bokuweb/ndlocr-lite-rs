# FVC v2を現在の本体へ接続する（実装前仕様）

本体はFVB。凍結v2の全件run39593424953（fe0e359）の全56 worker/79,020行、
入力別文字/語悪化0、全Page本文/bbox/f32 bits変化0の確認をruntime接続の前提とする。
新72と設定384の先行gateは完了している。旧v1の成功は代用しない。

先にRootへ既知121字列のCLI回帰を追加し、現在のRootからfresh buildした基準CLIで
DEIM off/onそれぞれのliteral本文/stdout LFが失敗することを確認する。
実argv、出力、stderr、終了codeを保存する。fixture/source/lock/model/実行物を固定する。
このRedの準備は全件待ちの間に進めてよいが、runtime変更は前提gate成功後に行う。

接続時は凍結sourceのruntime差分だけを移す。既存finish loop/initial batchは同じ処理を保ち、
採用vertical cropの初期認識を再実行せず、既存finishを一度だけ呼ぶ。
非採用時は既存経路へ戻り、finishのエラーはそのまま返す。padding/threshold/quality/cascade設定を維持する。
現在の32efの等価iterator整理は保持し、private capture moduleは本体へ追加しない。
アプリ用の純粋10契約とpadding契約を持ち込み、RootでRed→Greenを確認する。

既知quality回帰は、公開known packetと同一の12字PNGを固定fixtureとし、
既存full batchと採用vertical finishの本文/mean bits一致、およびforced delta=-100で初期結果からの
実変化を必須にする。通常の推奨設定を変更するための回帰ではない。
このnative回帰とCLI2件をLinux/macOSの必須モデルCIで実行し、環境変数の欠落によるskipはしない。
feature無効の振舞い、通常suite、ONNX suite/Clippyも確認し、source bridgeと全件証拠をPRへ保存する。
