# 本文1文字の分離した部品を支持用viewでまとめる

## 実装・追加認識前仕様

比較元はdb25206に `docs/single_pixel_body_bridge.md` の固定した変更を加えた製品処理。
この製品のsource/binaryは `tmp/closer-bridge/verification-command.json` に凍結され、
ページ検証も同じbinaryで継続している。本案はtest-onlyで先行し、その検証完了前には製品へ接続しない。

残る既知のraw正読後損失546組の一つは `geometry_round_japanese_suffix_maru40_png`。
3モデルともraw `情報)` が正読だが、単行処理は `情報` とする。
「情」の部品bbox `[23,27,44,47]` は主部品 `[14,9,47,47]` のbottomだけと一致し、
従来の4辺厳密包含ではまとまらない。左側の独立した短画線 `[9,17,13,29]` も残り、
本文2字とcloserを5成分と数える。元画像・collector・NN・文字列は変えず、次の支持だけを診断する。
これは既知の合成開発例であり、未知資料holdoutではない。

- 対象は3 Unicode scalar、端の既存5 closer `) ） 」 』 】` が1個、本文2字はともに表示幅2。
  空白/語中closer/他の文字数は対象にしない。既存fallbackが成功したらその出力を保持する。
- RGB和660・8近傍・既存面積フィルタを維持する。filtered成分は4〜7個。
  text順の端の1成分だけをmarkとし、残る3〜6成分を本文候補とする。
- 全本文成分のunion bboxの高さをHとする。markとの水平gapは0〜max(1,floor(H/12))。
  mark幅はceil(H/4)〜floor(H/2)、高さはceil(H/2)〜floor(3H/2)、完全に塗ったbboxは除く。
- 本文bbox内の全前景画素数が本文全成分の面積和と一致すること。無視された内部の点を混ぜない。
  本文bboxを横切る前景0の列のうち、水平中心に最も近い列を選び、同距離なら左を選ぶ。
  各成分全体がその列の左か右へ割り当てられ、左右とも非空であること。
- 左右のbbox間のgapは1〜max(1,floor(H/12))。各字形unionは高さceil(3H/4)以上、
  幅ceil(3H/4)〜floor(3H/2)。主部品は高さ最大・同高さなら面積最大・同値なら左を選び、
  高さceil(3H/4)以上。他の部品のyは主部品内に収まり、水平bbox間gapもmax(1,floor(H/12))以内。
- 少なくとも一方の字形内に、主部品の3辺に厳密包含され、残る1辺だけが一致する部品があること。
  この内部部品の幅/高さはともにceil(H/2)以上。4辺strict、2辺以上の一致、同bbox、細いdotは対象にしない。
- 主部品/他部品のbboxと面積を合算した2字形と、元markの計3成分を既存 `supports_collected` で確認する。
  支持されてもfragmentをそのまま保持するだけ。本文/closerの種類やASCII空白は推測しない。
  単行の最後のfallbackだけを候補とし、fragment/raw/confidence/追加forwardは変更しない。

## Red → Greenと採用条件

まず3辺strict/1辺一致と大きな内部部品の純粋判定をRed→Greenにし、次に字形のgroupと実画像を扱う。
各境界、同bbox/2辺一致/4辺strict/細点、sidecar gap、本文内空白gap、markの塗り潰しとgap、
無視された内部点、不正RGB/寸法/文字列、元RGB不変を例示する。
PBT各64例で白紙追加/平行移動、閾値未満の灰度、整数block拡大1〜3倍を検査する。
拡大は二値の支持maskに対する性質で、NNや任意の補間拡大の精度不変を主張しない。

最初に既存の保存raw33,735組（旧closer/空白/前段新規Auto・Horizontal）を実Rust基準/診断で比較する。
非悪化の場合だけ次の追加画像を生成・認識する。いずれの段階でも1指標の退行があれば本案を棄却し、
新結果を見て同じ対照へ閾値を合わせ直さない。

新規の本文は `稲穂` / `樹枝`、同じ3font file、22/38px、palt=0/1。
本文2字間ASCII空白0/1/2、端closer5種類・先頭/末尾、そのcloserと本文間ASCII空白0/1/2、
上下紙0/8px、PNG/JPEG75の8,640入力を認識前に固定する。
本文2字間空白も含む同じ本文に、本文だけ/細縦線1px/幅ceil(H/4)縦線/4×4点を
先頭/末尾・gap0/1pxで配置する2,304負例も固定する。合計10,944画像。
形状と本文の重なり0・元画素保持を保存し、CoreText file/glyph/advanceと全SHAを先に検査する。
Auto回転は除外せず、元Autoと有効Horizontalの前処理tensorを3モデルでbit照合してから全ROIを1forwardで認識する。
Autoが回転した画像は元のHorizontal指定も別途監査する。元方向の画像を支持に用いる。
全群を正式toolのnone/width/strictで文字・単語とも比較し、非悪化の場合だけ公開APIのRedから製品接続へ進む。

## 最終検証の記録

判定条件を変更せず製品接続まで完了した。全件結果、適用対照の限界、
実NN/API/CLIと再生の区別、再現手順は
[detached-body-components](benchmarks/detached-body-components/README.md) を参照。
凍結時点の先行仕様は同packet内の診断archiveにも保存する。
