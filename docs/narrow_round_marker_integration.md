# 狭いround profileの製品接続条件

## 追加NNの結果を見る前の仕様

比較元1ec730aと凍結したtest-only候補を比較する。追加8,064条件と元Horizontal指定244条件を
全3モデルで測定し、既存137,991組と合わせて162,915組を保持する。
controls仕様の全群・全正規化の文字/単語非悪化、および未重複RGBへの実NN適用を必須とする。
不成立なら本番には接続せず、反例と凍結した条件を保存する。

成立した場合だけ、まず公開の行認識経路で失敗するテストを追加する。
既存のstrict marker経路を保ち、その後に同じ本文所有関係と凍結round profileを使う
単一fragment救済を追加する。文字列の単語・文書種別で分岐しない。
元画像、方向、raw、fragment、平均confidenceの意味を保ち、追加のNN forwardは行わない。
全162,915保存raw組で実製品出力と候補出力が一致することを確認する。

実Session/CLIの検証対象は結果による選別を避け、以下で定める。

- 変更する全モデル/条件/方向組を実Sessionで確認する。
- 追加画像の全noiseに対して、本文内空白数+gap+JPEG指標+先頭指標が偶数のIDを実Sessionへ加える。
- 文字画像の本文内空白と端空白がともに1、紙2pxのIDを実Sessionへ加える。
- 元Auto回転する244入力はすべて元Horizontal指定で実Sessionへ加える。
- CLIにはAutoで変更する全組と、noiseの(先頭指標,gap,JPEG指標)が
  (0,0,0)または(1,1,1)の全ID×3モデルを加える。
- CLIが方向指定を持たない場合、Horizontal専用組は実Sessionで検証する。

同一対象は重複実行しない。実Sessionでは元RGB不変、rawと最終本文、fragment、平均confidenceの
float32 bit一致を検査する。CLIは出力末尾のLFをちょうど1個だけ除いて比較する。
これらの実Session/CLI forwardは追加評価用24,924 raw forwardと別に数える。
