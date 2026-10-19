# 分離した本文部品と語中の閉じ括弧

先行仕様は [internal_detached_closer.md](../../internal_detached_closer.md)、公開接続仕様は
[internal_detached_closer_integration.md](../../internal_detached_closer_integration.md)。比較元は0b7595c。
モデルの正読 `情)報` を単行後処理が `情報` にしていた損失を、左右の本文部品と中央の独立markで支持する。
空白のない3 scalar候補、中央closer5種の1個、左右の表示幅2文字だけを対象とする。
本文groupは前段の条件を再利用し、3辺strict包含/1辺共有の大部品、全前景の所属、幅/高さ/gap、
曖昧な複数markの拒否を確認する。単行最後のfallbackだけへ接続し、元RGB/前処理/NN/raw/候補/fragment/meanは維持する。

## 固定した全件比較

| 群 | 画像数 | モデル数 | 本文変更 | 生完全一致回復 | width完全一致回復 | 文字/単語指標退行 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 旧closer | 2871 | 3 | 9 | 9 | 9 | 0 / 0 |
| F/G空白対照 | 4598 | 3 | 0 | 0 | 0 | 0 / 0 |
| 1画素案の新対照 | 3648 | 3 | 0 | 0 | 0 | 0 / 0 |
| 上記の元Horizontal指定 | 128 | 3 | 0 | 0 | 0 | 0 / 0 |
| 前段の稲穂/樹枝 | 10944 | 3 | 0 | 0 | 0 | 0 / 0 |
| 前段の情人/情理 | 10944 | 3 | 0 | 0 | 0 | 0 / 0 |
| 上記の元Horizontal指定 | 64 | 3 | 0 | 0 | 0 | 0 / 0 |
| 今回の情天/稲木・内側空白/非文字負例 | 5088 | 3 | 127 | 80 | 112 | 0 / 0 |
| 上記の元Horizontal指定 | 56 | 3 | 0 | 0 | 0 | 0 / 0 |

全群をnone/width/strictの文字・単語距離で全件比較した。115,023組中136組の本文が変わり、
生完全一致89組・width完全一致121組を回復した。131組は何らかの指標が改善し、
5組は全正規化の全指標で同点。136組すべてを改善と数えず、文字距離の改善も全文一致と区別する。
旧raw正読後の損失は520→511組となった。前段の別baselineによる改善数へ合算しない。

新対照は3font file・24/40px・palt=0/1、5種類の語中closer、左右独立のASCII空白0/1/2、
上下紙0/8、PNG/JPEG75の4,320画像と、本文のみ/細線/太線/点の768負例。
本文`情天`は既知の情部品を用いた適用対照で、未知字形/font/実資料holdoutではない。
組合せ軸を変えず生成前の算術誤りを訂正し、初回の白列なしによる生成停止も保存した。
NN前に、負例の切断列は白列優先/なければink bbox中央と追記した。後者は接触画線を分離しうる合成負例。
本文全非白画素を左右の部分へ保ち、shapeとの上書き0を384 pre-JPEG組で独立再構成した。
生成失敗の入力は追加NNや評価の標本へ数えない。

データ計測の全ROI forwardは15,264 + 168 = 15,432で全件正常終了。
Auto回転56画像も除外せず、全5,088入力の元Auto/有効Horizontal tensorを3形状で先にbit照合した。
元Horizontal指定は同じ原画像の別条件であり、独立標本へ数えない。
実30モデル入力RGB 5,144件、native renderer 1,104記録のfont file/glyph/advanceと全SHAも確認した。

## 製品接続と公開経路

合成支持のRed→Green、実画像・境界・内部点・塗り潰し・曖昧な2候補と64例PBTを先行した。
旧新の全件非悪化と追加NNでの適用を確認してから、純粋な製品処理と実SessionのRedを記録し、
最後のfallbackへ最小接続してGreenとした。3モデルのSession/Pool/全batch/standalone、
raw/候補/fragment、RGB/mean bit、エラー後再使用、Cascade開始3/2/1も実モデルで成功した。

凍結実製品binaryの115,023組は事前案と全件一致。保存rawの再生であり、新しいNN forwardではない。
全変更136組に加え、結果を使わず固定した384負例・240空白入り本文・元Horizontal56画像を
全3モデルの実公開APIで再認識し、全2,176組のraw/本文/f32 mean bitが一致した。
全変更136組と固定192負例×3モデルの実単行CLI 712件も全件一致。
stdout終端LFを厳密に1個だけ除き、空白や他の改行をstripしなかった。
データ計測と公開経路の再認識、回帰スイートの認識回数を混同しない。

通常661件・ONNX CI対象593件・example23件が成功し、新規Clippy警告0。
環境変数なしでno-opとなる診断もテスト件数に含まれる。
今回の新規ページNNは0。fragment用poolへの呼び分け、関係4ファイル、cachedの新test-only import以外、
fragment policyと元の全helperのbyte一致を0b7595cと確認した。
以前の287ページ×8経路の測定は以前の証拠であり、今回の新規測定へ合算しない。
未知実資料での改善率と速度は未検証。

## 再現

4archiveは原画像/RGB/truth/実NN入力、全NN rawと上位2候補の確率、全採点・API・CLI結果、
凍結source/binary/model/input SHA、コマンド、成功/失敗とRed/Greenを含む。
`parents.json` は前段6archiveとそのnested parent 3archiveをSHAで参照する。
各indexはarchive/memberのSHAを持ち、aliasesで重複を可逆に復元する。モデル/font/実binary本体は含めない。

```sh
python3 docs/benchmarks/internal-detached-closer/reproduce.py --output tmp/internal-detached-restored
```

Pillowが必要。全archive/member/aliasのSHAを検証・復元し、正式scorerの162表ファイルをbyte再現、
3監査、実API/CLI/mean bit、元画素/実入力、native traces、fragment経路のsourceも照合する。
`product-manifest.json` を `NDLOCR_INTERNAL_DETACHED_PRODUCT_MANIFEST` に指定して
`verify_product_matches_frozen_internal_policy` を明示実行すれば、モデルなしで全製品方針を検査できる。
`tensor` を `NDLOCR_INTERNAL_DETACHED_ROOT` に指定して `verify_fresh_internal_auto_tensors` を実行すると、
新対照の前処理bit一致もモデルなしで再計算できる。
独立復元と実行SHAの結果は `verification.json` に記録する。
