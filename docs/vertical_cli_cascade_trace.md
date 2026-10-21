# 本体OのCLIモデル選択trace（既知3入力、方針変更前）

保存Page診断で完全prefix差が見つかったadvance_bold_71_pad8、composite_arial_67_pad0、
advance_serif_83_pad0を、既知の原因調査用入力として固定する。未知資料の改善とは数えない。
本体O/1f240beの凍結sourceを基準とし、FVC・追加実資料それぞれの固定gateを維持する。
この独立診断は、本体Oの既知入力に対するCLI観測として実行する。

隔離trace版は既存cached batchを一度だけ呼び、返り値をそのまま返すwrapperを追加する。
initial30/50/100、retry50/100、cascade-offの選択poolの実input寸法、入力index/bbox/bucket、
元crop RGB、本文とf32 confidence bitsを記録する。初期cascade後とpage batch後のsnapshotも分ける。
NN kernel/crop/判定/モデル順序・閾値は変えず、pool APIやsnapshotと内部forwardを区別する。
内部forward/model session loadが未計測ならnull。traceは新しいdirectoryだけに保存する。

純粋testを先行し、無効時の透明性・callback一回/エラー保持、RGB/本文/bitの記録、snapshot、
古い記録の非上書きと任意phase文字列のpath隔離をRed→Greenにする。
実認識前にsource/Cargo/model/charset/PNG/GTと実行物を固定する。

既定設定、単一model30 override/cascade off、単一model50 override/cascade offの3設定を、
3入力×通常O/trace Oで比較する（18 CLI process/18画像結果）。すべてDEIM on、他の設定は既定。
overrideでは任意30/50モデルに未存在pathを明示する。通常/traceのexit/stdout/stderr/bodyが完全一致し、
実crop RGBが元PNGのbboxと一致することを必要とする。observerが追加NNを要求することはない。

追加の呼出しはこの18 processだけとし、Session/内部forward数は推測しない。
公式none/width/strictと独立指標を全54 CSV行で照合する。通常/trace一致と観測の完全性を検査し、
既知の欠落が再現しなければ再現しなかったと保存する。最初から100を選ぶのか、promotionで長い結果を
失うのかを区別し、Page代理観測だけから原因を断定しない。実行は同じLinux runnerで両sourceをfreshに
コンパイルして行う。過去の通常/trace結果やMac予測は流用しない。

得られたtraceは診断であり、モデル選択の新policyは別仕様・Red/Green・未測定対照・全件gateを必要とする。

評価時のthread環境は既存Linux matrixと同じ`NDLOCR_INTRA_THREADS=1`とし、
`NDLOCR_PARSEQ_PARALLELISM=1`を明示する。設定は全18 process共通で、モデル・判定閾値は同じ。
既定CLI argvは既存matrix.cli_commandと完全一致させ、単一モデルoverrideのargvだけを先に純粋テストする。

最初のLinux attemptはobserver純粋testのコンパイルで停止し、CLI認識0だった。
凍結sourceに含まれる既存include_bytes!が要求するjma-notice-body.pngをpacketへ入れていなかった。
推論source・モデル・3入力は変えず、1f240beの同じcompile-time fixtureを両sourceへ追加してSHAを固定する。
元bundle/source archiveと失敗ログは別名で保存し、欠けたfixtureを検知するreader回帰を先にRedにする。
