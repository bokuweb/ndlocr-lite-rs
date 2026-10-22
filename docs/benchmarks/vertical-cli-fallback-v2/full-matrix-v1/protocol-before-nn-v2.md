# FVC v2の全既知gate（全件v2 NN前）

v2新72の全576結果/1,728 CSV行で、全入力別文字/語非悪化と新本文raw改善/語非悪化が成立した場合だけ実行する。
非default設定も別に必要で、v1全件run39568828241の成功をv2 sourceの成功へ流用しない。
v1 packet、source、失敗、partial、結果は元のまま保存する。

v1全件の1,239物理/1,245採点IDに、v2新72の同じPNG/RGB/GTを追加する。
計1,311物理ID/1,317採点ID/1,310 unique RGB。旧重複RGBと実資料の全GT aliasを保持し、独立標本数とはしない。
原の各origin/pathを保持し、新72だけ別originへ割り当てる。既知群のoriginを一つに上書きしてはならない。
ID/alias衝突、入力欠落、未採点、未知physical参照、v2新72の不足や旧RGBとの重複を拒否する。

基準FVBと固定v2 sourceを同じLinux/releaseでfresh buildし、CLIとPage実行物を凍結して56完全shardへ配る。
CLI DEIM/bands×cascade on/offは前後10,488 process、Page30/50/100×eval/defaultは前後15,732 API結果。
計26,220画像結果/79,020公式CSV行を独立距離と照合する。前後比較は39,510行となる。
Pageの全本文/行数/bbox/f32 confidence bitsは完全一致、CLIは全入力/設定/GT alias/3正規化の文字/語非悪化を要求する。
新72の改善を二度新規へ数えない。Page load48/明示model session96という計画と、内部forward未計測nullを区別する。

7 kind×8 shard、完全Cartesian、planned argv、mixed PNG/JPEGの元拡張子保持、partial除外、重複/別build拒否、
元source/lock/models/charset/画像/GT/実行物/harnessの前後SHA照合には既存の検証済みfull runnerを再利用する。
既存runnerや実行中v1 harnessのbyteを変更せず、v2 bundleの読込/union/新72 gate検証を別wrapperへ置く。
wrapper自体とreaderもharness receiptへ追加する。rootの未確定sourceや共有target実行物を採点へ使わない。

before-NNに全scope/source/planを凍結し、model-free union/alias/format/網羅契約をRed→Greenにする。
新72 gateが未成立ならwrapperの認識/build dispatchへ進めない。
全件成功後もrootへの最小source bridge、既存等価iterator整理との対応、必要suite/実モデルCIを確認してから本体接続する。

## 最終件数検証の限定修正

共通runnerに旧74,700行の固定assertが残っていたため、最初のv2 run39591981411をbuild段階で中止した。
そのrunのmatrix/compareは未実行でNN0。`docs/vertical_cli_fallback_v2_metric_count.md`を先行し、
現在の共通runnerの最終件数確認だけをplanとの完全一致へ一般化する。旧1f7 event/harnessは元のまま保存する。
認識/採点/悪化判定の関数byteは不変、差は追加した件数validatorと末尾1呼出しだけと照合する。
元protocol-before-nn.mdを保持し、再実行はprotocol-before-nn-v2.mdを別SHAで指定する。
