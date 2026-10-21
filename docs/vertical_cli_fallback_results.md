# FVCの新72 CLI先行gate

事前固定した72画像をDEIM/bands×cascade on/offで基準FVB・隔離FVCの前後測定し、
576 CLI process/画像結果と公式none/width/strictの1,728 CSV行を完了した。
独立距離による全誤り数/分母が一致し、288入力/経路組の全3正規化で文字/語の悪化は0だった。
source/lock/model/charset/PNG/RGB/GTと異なるfresh実行物のSHAは認識前後で一致した。
CLI内部forward/session数は未計測nullで、プロセス数から推定しない。

14本文画像のraw30経路組が改善した。bandsのcascade on/offは各14、DEIMはon/off各1。
36本文のうち残る画像と、反復12/縦長横書き12/非文字12は全経路で本文が不変だった。
例えばarial43/pad4のbandsは、生文字誤り39→0（cascade on）、40→0（off）。
既知fontの合成開発対照であり、未知資料の改善率や30の独立画像とは主張しない。

先行する既知arial121のCLI試験は旧版のliteral本文Red1→候補Green1。
候補はDEIM off/on両方でbodyとstdoutの末尾LFを保持したが、単一30 override/cascade offの既知例であり、
新72の改善数へ数えない。feature無効16 processの比較や、失敗したtest期待の旧履歴も維持する。

全結果は`docs/benchmarks/vertical-cli-fallback/new72-cli-matrix-v1.*`へ保存した。
readerは全2,406 member、既存before-NN packetとのsource/入力対応、実build/argv/body framing、
全1,728公式指標行と864前後比較の全内容をモデル不要で再検算し、Red→Greenとなった。
これは新72の先行gate成功であり、本体統合や全既知gateの成功ではない。

次の全件は、認識前に追加されたC1新96も入力として含む1,239物理ID/1,245採点ID。
C1の推論方針は混ぜない。全10経路・前後24,780画像結果/74,700 CSV行、
Page本文/矩形/confidence bits不変、設定別対照・必要suite/CIを通過するまで本体はFVBを維持する。
