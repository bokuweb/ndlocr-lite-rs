# FVB全件比較の独立CI実行（追加NN前仕様）

ローカルの追加270 baseline/570 candidateは、容量枯渇時に両driverがexit120で終了した。
残るtextファイルはbaseline30/eval251、candidate30/eval38、candidate50/eval35で、少なくとも
324回の追加成功API返値があるが、child終了値・設定全JSONは未保存。全件結果とは扱わない。
同時のスワップ使用は約23GBだった。原因を一つに断定せず、partial/失敗/計画を保持する。

全件gateは新たにGitHub Actionsの独立したLinux環境で実行する。以下は資源/検証環境の変更で、
FVBの幾何・出力支持・confidence・画像・GTを変えない。Macの348結果をLinuxのbaselineや
candidate結果へ流用しない。全件のsource/data/model/charset/lockと実行物を固定して再測定する。
本体推論は1f240beのままとし、候補は凍結source snapshotからだけbuildする。

## パケットとsource

- 既知918のPNG/JPEG等とGT、およびRashomon3物理crop/4評価GTを内容SHA付きで保存する。
  NNに渡す入力IDは921（同じRGBの旧ID重複を保持）、評価IDは922、unique RGBは920。
  評価IDと物理prediction IDを明示的に結び、GT二表記でNN回数を水増ししない。
- baseline sourceはGit1f240be。推論runtimeは測定済みO/73e9ab4とtest-only差。
  candidate sourceはMac先行比較のFVB source全byteと一致させる。モデル/フォント/native binaryは
  パケットへ入れない。Cargo.lockはsnapshot archive内に置き、rootのignored状態を保つ。
- producer/readerを先にテストし、SHA不一致、重複/欠落ID、危険なmember名、モデル/出力結び付き、
  fresh=trueや同じPage実行物の取り違え、partial結果を拒否する。
- source snapshotからrelease/ONNXをbuildする。baseline完成後にcandidateのRust mtimesを更新し、
  source bytes不変、lib/CLI/Page exampleのfresh=falseを検査する。コピーした実行物SHAを固定する。
  Page exampleは前後で異なるSHAが必要。CLIは同じSHAでも全入力を両variantで測定する。

## 独立runnerと件数

Page30/50/100とCLIを別runnerの4 jobsに分ける。各runnerではNN processは1つずつとする。
各Page jobはbaselineとcandidateを直列に、921入力×eval/defaultで認識する。
全Page実測は921×6×2=11,052外側API結果、各variant/model1loadならPageRecognizer6load /
model session12load。内部forward/候補内エラーは未計測null。失敗した場合は実績を別記する。
CLI jobは921×DEIM/bands×2variant=3,684プロセスを直列で実行し、load/forwardはnullのまま。
raw stdoutと本文+LF1個を検査する。全失敗とpartialをartifactへ保存し、成功と扱わない。
release/Linuxは別の新規検証環境なので、Mac/debugとの本文差をFVB差分として数えない。

## gate

4 jobsの実行物/source/input/model bindingsと全件数を検査した上で、公式scorerと独立距離を
全922評価ID×8経路×none/width/strictで照合する。44,256 CSV行と44,256文字/語比較を予定する。
各入力の文字/語が1件でも悪化したら全FVBを保留する。新規FVB多様本文でraw文字改善と語非悪化が
少なくとも1件必要。RashomonはどちらのGT表記も非悪化が必要で、独立資料数は1と記載する。
CLIは変更対象callgraph外なので本文の完全一致も要求し、差があれば理由を調査する。

全件gate、実PageのO Red/FVB Green、source対応、本体既存suite/CIが揃うまで本体へ採用しない。
CI用workflowは読み取り権限と通常runnerを用い、入力/source/driverの変更時だけ起動する。
結果だけの文書更新で長時間matrixを再実行しない。
