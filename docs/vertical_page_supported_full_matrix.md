# FVB全件gate（追加NN前仕様）

348入力×Page6の先行比較は、公式/独立12,528 CSV行の一致、悪化0で通過した。
本文変更33入力/経路組はすべて文字又は語改善、新108内の本文5画像・7経路組でraw文字改善と
語非悪化があった。これは採用ではなく、以下を全て必要とする。

## Pageの全918入力

O基準版は既知444/FVA新96/FVB新108を再利用し、残り270を別の事前仕様で測定中。
FVBは同一の固定Page実行物31357f581bc21f482adab47d60b3ad9f0ed532962bb4981cc7f96b81c165f9c9を使い、
先行比較のcandidate348をそのまま再利用し、残り570を全3model×eval/defaultで実測する。
幾何で非該当の入力も除外せず、新たなPage結果は570×6=3,420を予定する。
追加PageRecognizer load3、model session load6、内部forward/候補内エラーは未計測null。
同一source/input/model/optionsの再利用であり、native結果や違うcandidateの結果をPageへ流用しない。

## CLIの全918入力

CLIはCachedPageRecognizerでPage APIとは別callgraphなので、別の検査とする。
O基準の正しい実行物SHA575bdd38c9b6bd9ce74395035b0b0baf9818b2b3b8b2b06267ff0f7274c69c43と比較する。
候補CLIを同じFVB source・lockからfresh=falseでcompileし、SHA/モデル/charset/コマンドを固定する。
実行物が同一SHAなら既存と新規baselineの結果を再利用できる。異なるSHAならcandidate全918を
DEIM/bands両方で実測する。基準CLIは既知444を再利用し、残474×2=948プロセスを実測する。
新candidateの場合は918×2=1,836プロセス。CLI load/内部forwardは未計測nullにする。
raw stdoutは保存し、出力本文+transport LF1個と一致することを要求する。本文/GTをstripしない。

資源上、追加Pageの並列度は2、基準270のPageは1。CLI基準474は1で実行し、基準270と
CLI基準474が両方完了してから候補CLIを最大2並列で実行する。local推論プロセスは最大4に抑える。
モデル付き回帰を並行する場合もこの上限に含める。失敗・中断・再試行は保存する。

## 判定・本体

全918×8×none/width/strictで全入力の文字/語誤りを公式と独立距離で照合し、1件の悪化も拒否する。
別資料Rashomonの3物理crop/4GT casesは追加gateで、既存918を減らさない。Page36結果は取得済みで
全18物理入力/経路組の本文・矩形・confidenceが基準と同一だった。公式/独立評価とCLIも必要である。
同一RGB/GT異体表記を独立な資料数やNN呼数へ加算しない。

実際のPageRecognizerの121scalar本文fixtureで、Oが失敗しFVBが成功する回帰を先に作る。
raw GTのU+3000一字はcandidateも認識できていないため隠さず保持し、非空白本文全体の復元と
raw返値の保持、元buffer、eval/default、不正RGB後の再利用を検査する。GTやNN出力の空白は補正しない。
検証済みsourceとのbyte対応、必要な既存suite/CIとこの回帰が揃ってから本体へ接続する。
