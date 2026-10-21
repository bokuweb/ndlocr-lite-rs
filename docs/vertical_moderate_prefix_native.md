# Rの隔離実装と先行native gate

Root 0a0f421のCI39627691771は再実行attempt2で全5 jobs成功。初回の両モデルjobは
asset取得timeoutでNN前に失敗し、モデル/hash不変で再実行した。モデル不要CI39627691943も成功。
Linux/macの保存CLI4出力はliteral120 scalarとstdout LFに一致し、Root source852も固定値のまま。
これを前提にRの隔離実装を開始する。本体Rustは変更しない。

baseline/candidateをRootの全852 file/resourceから別コピーし、before SHAを記録する。
同じignored native collector（72入力×8 API）を使い、GTをrecognizerのmanifest/argvへ渡さない。
先に両sourceとfresh実行物を固定し、新72の予測を見てから条件を調整しない。

## 小さい実装順

1. 既存Oのpart読込loopだけを共通helperへ抽出する。先にhelper未実装Redを作り、
   両経路の順序・元RGB座標・各part一度だけ・最初のerror停止・不正cropのcallback0を検証する。
   Oのplans/combine/既存pure/PBTを維持する。
2. Rのraw17..=T-2とanchor `max(40,2*raw_count)`、不明T/overflow/不正入力の事前拒否を
   Red→Greenで実装する。raw confidenceも有限0..=1の有効な結果に限定する。
   事前条件不成立はcallback0。part読込後のNaN/空/不一致/非prefix/空白変更は棄却し、
   既に実行したcallback数を0と偽らず記録する。
3. Oと共通の画像支持/二分割/合成を使う。Oのcapacity範囲とRの範囲は明示し、
   Unicode全文prefixと空白scalar保持には例示とproperty testを置く。
4. cached Fragment AutoのH/O不採用後だけに接続する。同じ借用sessionを使い、
   static/実frame整合、既存sanitizeの位置、他5 APIとbatch/専有batchの一致を守る。

## 採否

対象3/非対象5の訂正は `vertical_moderate_prefix_api_scope.md`。
新72は前後3,456結果、10,368公式CSV、1,728 raw比較/5,184正規化比較、outer API2,604。
明示loader18、内部forwardは未計測null。全入力文字/語悪化0、非本文全本文不変、非対象5の
本文/mean bits不変、単件/batch/専有batch一致と、新body Fragment Autoのraw改善を必要とする。
改善0または悪化ならR全体を保留し、この72に条件を合わせ直さない。
成功時だけ、全既知native/Page/CLI gateを追加NN前に固定する。本体接続はその後である。
