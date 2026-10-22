# FVB: 単縦列fallbackの出力支持（実装前仕様）

FVAは採用しない。30/evalの先行240入力で、非文字brush40の空出力がconfidence
0.985090434551239の「AAAA」へ変わった。none/width/strictすべて文字0→4・語0→1。
他モデルの測定と全公式照合は完了まで続ける。FVAのsource/条件/結果は変更しない。
以下はこの既知反例に基づく別案であり、FVAの未測定対照と呼び直さない。

## 固定する変更

FVAの幾何・準備RGB・認識方法・confidence・失敗時の水平fallbackをそのまま維持し、
認識出力の支持を二つ追加する。文字の訂正や削除、特定文字列・文書種類の例外は作らない。

- 幾何判定で既に数えているsquare anchor数を、同じ走査から受け取る。
  閾値220、run長と横断extentがともに短軸のceil(3/5)以上という定義は変更しない。
- 出力の非空白Unicode scalar数がceil(anchor数/2)以上であることを要求する。
  confidenceが高くても、多数の支持字形に対して短すぎる出力は従来経路へ戻す。
  空白を増やしてこの条件を満たせないよう、空白scalarは数えない。
- 出力に4種類以上のUnicode alphanumeric scalarを要求する。
  両方の数値条件は既存Hの`vertical_capacity/policy.rs::combine`にある
  groups/2・4種類の支持を参照する。H/O自体は変更せず、FVBだけへ適用する。
  画像anchorは文字の厳密な個数ではないため、等数一致や文字の置換は要求しない。
- 採用時は元のtext/confidence/bboxをそのまま返す。棄却時はFVAと同じ水平fallback。
  この検査のための新しいNN、再load、再認識は追加しない。

正当な繰返し文も新経路からは棄却されうる。従来結果を保持することを対照で検査する。
「AAAA」だけを拒否する回帰修正とはしないし、この条件だけで非文字判定を保証しない。

## Red → Green → Refactor

先に、12anchorに対して非空白5/6 scalarの境界、3/4種類の境界、十分な長さの繰返し、
空白による水増し、CJK/非BMP scalar、confidence/失敗/元buffer/1 callbackをテストする。
FVAの幾何全記録は不変を要求し、metadata抽出のrefactor前後でも810入力×raw/preparedを照合する。
既存のhelper positiveは新しい支持を満たす本文へ明示的に更新し、元FVA試験は履歴に残す。

## 新規108対照（FVB追加NN前に生成条件・本文・font SHAを固定）

- 多様本文60: 既知3font、(font24/advance28,font36/advance44)、padding2/10、
  長さ12/31/51/81/121。一つの新しい本文の先頭N scalarを描画する。
- 繰返し本文12: 同じ3font×2size×2padding、81 scalar。fontごとに一文字の
  「一」「A」「口」を繰り返す。非文字GTにせず正当な繰返し文の保持対照とする。
- portrait横書き12: 同じ3font×2size×2padding、新しい8行の複数字word。
- 非文字24: white/warm/gray×4種×groups8/40。
  alternate_brush/tapered_zigzag/paired_blocks/diagonal_marksを固定手順で描く。
  empty GTは描画した非文字という診断規約であり、人間にも必ず文字に見えないとは主張しない。
- 全PNG/RGB/GT/glyph BBox、独立getmask描画、重複検査を固定する。
  既知810へ加えた全件をraw/prepared幾何検査し、全非白画素・glyph BBox切断0を必要とする。

## モデルgate

追加NN前に実件数/unique RGBを固定する。先行比較はFVA既知240と新規108、予定348×6
Page API経路。FVA候補はbaselineに使わず、FVAで固定したO baselineの240結果を再利用し、
新108 baselineと全348 candidateを新規測定する。baseline/candidate source・実行物・モデル・
charset・lock・入力を固定し、候補Page実行物のfresh=falseと異なるSHAを検証する。
失敗も保存し、内部forward/飲み込んだ候補NNエラーが未計測ならnullのままにする。

none/width/strictの文字・語誤りを公式/独立で照合し、1件でも悪化すれば全FVBを保留する。
新規多様本文での文字改善と語非悪化を最低1件要求する。先行成功は採用ではなく、採用には
全入力（予定918 ID）×Page6/CLI2の比較、実PageのRed/Green、必要なsuite/CIを必要とする。
CLIの別callgraphと同一実行物の再利用条件はFVA仕様の補足に従う。
