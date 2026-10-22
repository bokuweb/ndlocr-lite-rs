# Root接続前の回帰準備

Root runtimeはFVBのまま。現在のsourceからONNX lib/CLI/新integration testをfresh=falseでbuildし、
CLI（SHA256 `a00ebea4092a227ce3b2181b6eeb0854065dd5ef6866a2a5d05aeff87904c655`）と
test（`429a743b5a2b9be1a79ca6b53e631d88022d729fd0320919916059a6ceee7c77`）を別名へ固定した。
全844 source/fixture、lock、使う3 asset、実行物とdriverの認識前後SHAが一致した。

同じ既知arial121に対し、DEIM off/onのCLI自体はexit0だが、literal本文がいずれも360 scalarの
分断出力となり、新しい2テストは期待した本文assertでRedとなった（test exit101）。
実body/stdout/stderr/argvを個別保存した。凍結v2 CLIを同じtest binaryへ渡したknown precheckは2 Green。
GTは121字、literal期待値は既存の全角空白欠落を残す120字であり、どちらも元SHAと内容を保持した。
詳細は`docs/vertical_cli_known_literal_length_audit.md`。既知例なので新規改善数へ加算しない。

feature無効のRoot基準CLIも、同じ844 sourceでlib/binのfresh=falseとonnx feature不在を確認して固定した。
build段階のNN/model loadは0。接続後のfeature無効CLIと別途16 processを比較するための基準である。

公開`root-bridge-v1/baseline-before-green-v1.*`は185,964 bytes、
SHA256 `05a1a813cb7fde8d00b2031823c1b48723af155cbbd39f1464f24ae39dd23fae`。
843 sourceは既存geometry packetを参照し、新test1ファイルだけ追加する。
モデル不要readerの未実装Red、初回の字数assert誤り、訂正後7 packet tests Greenを保存する。
known qualityの12字fixtureと回帰sourceもRootへ準備したが、appへの登録・実行は接続時に行う。

全件run39593424953は引き続き凍結v2の評価として進め、成功とPage不変/入力別非悪化を確認するまで
Runtimeへ接続しない。追加CLI2テストは現在の本体では意図したRedで、通常はモデル必須のignoredである。
接続後にLinux/macOSの必須モデルCIへ追加してGreenを要求する。
