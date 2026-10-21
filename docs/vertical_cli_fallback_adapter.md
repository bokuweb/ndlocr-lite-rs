# FVCの隔離CLI adapter（追加NN前）

固定済みFVC仕様・新72対照を変更せず、隔離sourceのCLI接続を小さく進める。
先行仕様の「純粋callbackテストRed→Green→隔離CLI接続」に当たる段階である。
FVB全件gateの整理前は実モデル認識と本体接続を行わない条件を維持する。

- 共通helperのcallback cropをPreparedLineCropへ包み、縦のreading-axis bucket、元RGB、
  padding、公開bboxとconfidence bitsを保持する。初期cached batchへ渡す。
- 純粋テストで、採用したprepared crop/resultをそのまま返し、legacy callbackを呼ばないことを
  Red→Greenにする。不採用ならlegacyを一度だけ呼び、エラーも従来どおり返す。
- 隔離CLIはCachedPageRecognizerを一度loadし、認識済み結果を下流へ一度だけ渡す。
  既存認識初期batch/cascadeの実装と引数は変更しない。legacyのband生成/支持/padding/認識、
  後段context/dict/rule-pack/並べ替えの実装を保持する。
- feature-disabledでは新helperへ入らず既存経路を保つ。default/onnxのコンパイルと純粋suiteを確認する。
  これだけで実CLIの実モデルRed/Greenや回帰不在を証明したと扱わない。
- 純粋テスト・source snapshotを追加保存し、後続の実モデル前に別SHAで凍結する。
  既存公開pure-before-nn-v1.zipとモデルgateは変更しない。

実CLI回帰は既知FVB arial121 fixtureのliteral expectedを使う。
`--model`へ固定PARSeq30を明示し、cascadeをoff、30/50の任意モデルを存在しないpathにする。
これは既存単一モデルoverrideの条件であり、default100/cascade経路の成功を意味しない。
DEIM off/onの両方で、output-txtのbody bytesとstdoutの末尾LF1個を検査する。
構造規則はoff、dict/rule-packなし、padding0。モデルがなければskipせず失敗する。
この既知回帰は新72対照の改善件数へ数えない。実行はFVB gateの整理後とする。
比較時は`NDLOCR_TEST_CLI_BINARY`でbaseline/candidateそれぞれのSHA確認済み凍結実行物を指定する。
Cargo共用targetの可変実行物を前後で取り違えない。未指定の通常test harnessはCargoの実行物を使う。

feature無効時は既知arial121と白画像600×900、DEIM off/onの4条件を、凍結した前後の
default CLIで比較する。本文画像はfeature-disabled error、白画像は空body/stdout LF1個を
期待し、exit code・stdout/stderr・body有無/bytesを前後完全一致とする。モデルpathは存在しない
固定pathを使い、NN0で実CLIの分岐・エラー保持を検証する。この結果もONNX精度gateとは区別する。

feature無効比較の初回では前後8 processのexit/stdout/stderr/bodyが同じだったが、
本文のmissing-model時はfeature errorより先に`file not found`が返ったため、test期待が失敗した。
製品変更は行わず、次の固定比較ではmissingと存在するdummy model/正しいcharset YAMLの
2条件を明示する。missing本文はfile error、dummy本文はfeature error、白は両方で空成功。
前後2×画像2×DEIM2×asset2=16 processを比較する。初回の出力と失敗は上書きしない。
