# 分離した本文部品と端の閉じ括弧

先行仕様は [detached_body_components.md](../../detached_body_components.md)、比較元は0127fe8。
`情報)` の情の大きい部品が外側成分と下端だけを共有する場合に、モデルの正読を後処理が `情報` へ壊していた。
原RGBの8近傍成分を変えず、白列の左右にある部品を支持用の2字形へまとめる。
大部品の3辺が厳密に内側で1辺だけ一致すること、本文/markの高さ・幅・gap、全前景の所属を確認する。
空白なし3字、端closer1個と表示幅2の本文2字に限る。単行の最後のfallbackに接続し、
raw/候補/fragment、元RGB、前処理、confidence、追加NN回数は変更しない。

## 固定した全件比較

| 群 | 画像数 | モデル数 | 本文変更 | 生完全一致回復 | width完全一致回復 | 文字/単語指標退行 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 旧closer | 2871 | 3 | 32 | 26 | 30 | 0 / 0 |
| F/G空白対照 | 4598 | 3 | 0 | 0 | 0 | 0 / 0 |
| 前段の新本文・空白・ノイズ | 3648 | 3 | 0 | 0 | 0 | 0 / 0 |
| 前段の元Horizontal指定 | 128 | 3 | 0 | 0 | 0 | 0 / 0 |
| 今回の稲穂/樹枝対照 | 10944 | 3 | 0 | 0 | 0 | 0 / 0 |
| 今回の情人/情理適用対照 | 10944 | 3 | 362 | 282 | 354 | 0 / 0 |
| 上記64画像の元Horizontal指定 | 64 | 3 | 0 | 0 | 0 | 0 / 0 |

各群のnone/width/strictで文字/単語距離を全件比較した。計99,591組中394組が変わり、
生完全一致308組、width完全一致384組を回復した。384組は何らかの指標が改善し、
残る10組は全正規化の全指標で同点だった。394組すべてを改善とは数えない。
旧raw正読後の損失は546→520組。モデルが出したcloser自体を置換しない。

稲穂/樹枝の新対照では適用0だったため、判定条件を変えず、別の適用対照仕様を先行した。
各10,944画像は3font file、2サイズ、palt=0/1、本文内/外ASCII空白0/1/2、
5closer・先頭/末尾・紙・PNG/JPEG75と、本文のみ/細線/太線/点の負例からなる。
情人/情理は既知の情の部品に第2字を変えた適用対照で、未知字形/font/実資料のholdoutではない。
真値を仮rawとして与えた幾何のみの132適用は、NNの改善へ合算しない。

新規の全ROI認識は32,832 + 32,832 + 192 = 65,856 forward。全入力を保持し正常終了した。
Auto回転する64画像を除外せず、原Autoと有効Horizontalのtensorを全3形状でbit照合した。
両新セットの4,464 native renderer記録とfont/glyph、2,304 pre-JPEG負例の非重複画素、
実30モデル入力RGB 21,952件も照合した。Horizontal指定は同じ画像の別条件で、独立標本ではない。

前段0127fe8の1画素案も、同じ新NNを再利用してdb25206基準で別監査した。
稲穂/樹枝20、情人/情理4、元Horizontal4組の生完全一致回復、全正規化の文字/単語退行0。
この別基準の回復を今回の308組に加えず、新しいNN forwardとも数えない。

## 製品接続と再現

仕様・小さなテストのRedを先行し、実画像と公開SessionのRed→最後のfallback接続→Greenを確認した。
7純粋テストと64例PBTで、部品/幅/高さ/gap/内点/矩形mark/方向、紙/移動/灰度/整数倍率を検査した。
3モデルのSession/Pool/並列batch/単一Session batch/standalone、raw/候補/fragment、
RGB/mean bit不変、エラー後再使用、Cascade開始3/2/1も実モデルで成功した。
凍結実製品binaryの99,591組replayは事前方針と全件一致した。これは新規NN測定ではない。
全変更と境界負例/方向条件の1,882組を実公開APIで再認識し、raw/本文/f32 mean bitが一致した。
全変更394組と負例288組の実単行CLI 682プロセスは、終端LFを厳密に1個だけ除いた本文が全件一致した。

通常655件・ONNX CI対象582件・example23件が成功し、新規Clippy警告0。
環境変数なしでno-opとなる診断もテスト件数には含まれる。
今回はページNNを新規実行していない。page用fragment経路と関係する関数のbyte一致を0127fe8と確認した。
前段の287×8経路の新規測定は前段の証拠であり、今回の新規測定へ合算しない。
未知実資料の改善率と速度は未検証。

6archiveは原画像/RGB/truth/実NN入力、全NN rawと確率、採点・API・CLI結果、
凍結source/binary/model/input SHA、生成/認識コマンドとRed/Greenを保存する。
元33,735組の画像等は `parents.json` で前段packetの正確なSHAを参照する。
各indexにarchive/memberのSHA、aliasesに重複の復元情報を持つ。モデル/font/実binaryは含めない。

```sh
python3 docs/benchmarks/detached-body-components/reproduce.py --output tmp/detached-restored
```

Pillowが必要。前段3archiveと今回6archiveを検証・復元し、正式scorerの180表ファイルをbyte再現、
7監査オブジェクト、実API/CLI/mean bit、原画素/実入力を照合する。
復元先 `product-manifest.json` を `NDLOCR_DETACHED_PRODUCT_MANIFEST` に指定し、
`verify_product_matches_frozen_detached_policy` を明示実行すれば、モデルなしで全製品方針も検査できる。
`tensor-initial` / `tensor-positive` を `NDLOCR_DETACHED_ROOT` に指定し、
`verify_fresh_detached_auto_tensors` で各新セットの前処理bit一致も検査できる。
独立復元の結果と実行SHAは `verification.json` に記録する。
