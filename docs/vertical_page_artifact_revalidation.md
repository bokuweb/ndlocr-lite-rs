# OのPage実行物を再検証する

## 2026-10-21: 事前仕様と誤ったgateの撤回

P1の実入力記録で、bold31のOがPageでも採用されることが観測された。
前回のproduct-{baseline,candidate}-freeze.jsonを照合すると、CLIおよびPage exampleのSHAが両側で同一だった。
候補build logは全target fresh=trueで、baselineはfresh=falseだった。
したがって前回の「Oの444 Page ID全8経路は不変」は候補実行を証明せず、採用gateを撤回する。
記録器が精度を変えたとは決めつけず、元出力・失敗・誤った報告を履歴として保存する。

## 修復の固定条件

1. 製品sourceは73e9ab4のままとする。root全Rust sourceのmtimeを更新し、同じbyte SHAを前後で確認する。
   新しいnon-test library、CLI、capture_page_matrixをビルドし、fresh=false、baselineとは異なるbinary SHAを要求する。
   既存の凍結済み実行中P1 binaryや原測定を上書きしない。
2. candidate-v2として全444 ID（443 unique RGB）を全3model × eval/default・CLI DEIM/bandsの8経路で実行する。
   既存baselineはpre-Oであることをsource/binary履歴で再確認して使う。
3. 全input/route/none,width,strictの文字・語誤りを公式scorerと独立distanceで比較し、1入力でも退行すればOを保留する。
   text/bbox/linecount/confidence bitsの差も全件保存する。GT/rawはstripせず、CLIが加えるLFだけを区別する。
4. P1の全144 ×3×2 Page結果は、このfreshなuntraced実行物の結果と比較する。
   一致するまでは記録器の無影響を証明済みとしない。
5. 原報告はincorrect/stale artifactとして残し、README/results/PR/plan/reproducer/公開packetを訂正する。
   履歴として再現可能な同一baselineの比較を、製品Oの精度検証として再掲しない。

根本原因はコピー元の古いmtimeを引き継いだsourceと共有Cargo targetによるstale再利用である可能性が高い。
ビルド時間だけでは判断せず、JSON fresh flag・binary SHA・実モデルの既知positiveで実行物を検証する。
この修復で新しい政策・閾値・文字置換を導入しない。目標全体は引き続きACTIVEとする。
