# 接触主成分と分離部品案の公開接続検証

旧115,023組とFreeType BASIC対照7,632画像×3、元Horizontal24画像×3を固定案で比較し、
全群・既知/未重複RGB・none/width/strictの文字/単語指標が非悪化、かつ未重複RGBへの
新NN実出力の増分適用がある場合だけ実行する。支持条件は結果へ合わせ直さない。

- 既知 `情報」` fixtureで、純粋単行処理と全3モデルの実Sessionが `情報` へ落とすRedを残す。
  raw/候補/fragmentは `情報」` を保持する。テスト以外のsource/model/binaryを固定する。
- 最後の単行fallbackへだけ接続し、前段の条件と元RGB/NN/meanを変更しない。
  元Auto非回転/明示Horizontalのみを対象にする。実製品再生数は確定manifestから算出し、
  旧115,023 + 新22,896 + 元Horizontal72 = 137,991組の凍結案と一致させる。
- 公開APIは旧新全変更組に加え、新noiseの `(body_spaces + gap + JPEGフラグ + leading)` が偶数の
  全576画像×3、本文内空白1/外側空白1/紙8の全240本文画像×3、元Horizontal全24画像×3を確認する。
  ID/model/方向の重複を除き、全実raw/本文/f32 mean bitとRGB不変を照合する。
- 同じseedでSession/Pool/並列batch/単一Session batch/standalone、raw/候補/fragment、
  エラー後再使用、Cascade開始3/2/1を全3モデルで確認する。支持処理の追加forwardは0。
- 実CLIは旧新全変更組に加え、各本文/font/size/body空白/shapeに対し
  `(leading,gap,JPEG)` が `(false,0,0)/(true,1,1)` の全288noise画像×3を確認する。
  末尾LFを正確に1個だけ除き、本文空白も保持する。
- 通常/ONNX CI対象/example/fmt/diff/Clippyを確認する。新規ページNNはせず、
  fragment経路の関係source/関数byte一致を前段のページ実測とは区別して記録する。
- 全source/fixture/input/model/実binary、全失敗/0適用/生成記録もSHA付きで保存し、
  モデル不要の正式scorer表再現・RGB/atlas/native/合成画素の独立復元と実Rust全件再生を確認する。

## 最終接続結果

全3モデルでraw正読/処理後欠落のSession Redを先行し、最後の単行fallbackの接続後にGreenにした。
固定条件の全137,991組を実製品で再生し、公開API2,602組のraw/本文/mean bitとRGB、実CLI946件が全件一致した。
通常671件/ONNX608件/example23件、新規Clippy警告0。未使用import整理後の最終sourceでも全検証を再実行した。
none完全一致51組には既存RGBと重複する7組を含み、追加の未重複RGBでは30組を回復した。
新規ページNNは0、fragment関係source不変性を確認した。詳細と独立再現は
[owned-parts-body-bridge](benchmarks/owned-parts-body-bridge/README.md) を参照する。
