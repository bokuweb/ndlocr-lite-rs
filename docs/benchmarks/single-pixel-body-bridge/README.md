# 1画素で接触した2文字と閉じ括弧

先行仕様は [single_pixel_body_bridge.md](../../single_pixel_body_bridge.md)。比較元はdb25206。
本文2文字が1画素だけで接触した画像では、モデルが正しく読んだ `情報)` を後処理が `情報` にしていた。
元RGBと8近傍collectorを保ち、支持用マスクの1画素だけを省いて独立した2字形になる場合に、元のcloserを保持する。
対象は空白のない3字、端の既存closer1個と表示幅2の本文2字だけ。
字形の幅/高さ・独立性・closerの幅/高さ/非矩形性と0/1pxのgapを確認し、既存の支持条件でも検査する。
既存の単行fallbackが成功した場合はそちらを使い、fragment/raw/候補/confidenceの扱いは維持する。

## 接続前に固定した比較

| 群 | 画像数 | モデル | 文字/単語指標の退行 | 生の完全一致回復 | width完全一致回復 |
| --- | ---: | ---: | ---: | ---: | ---: |
| 既存closer | 2871 | 3 | 0 / 0 | 25 | 30 |
| F/G空白対照 | 4598 | 3 | 0 / 0 | 0 | 0 |
| 新本文・括弧・空白・ノイズ | 3648 | 3 | 0 / 0 | 6 | 18 |
| 上記のAuto回転128画像をHorizontal固定 | 128 | 3 | 0 / 0 | 0 | 0 |

各群をnone/width/strictで全件採点した。Horizontalの128画像は新しい独立標本ではない。
すべて既知fontの合成開発対照であり、未知資料の改善率とは扱わない。
変更48本文のうち生文字/単語距離が改善した31本文は完全一致へ回復した。
残り17本文はモデルが出した半角closerをそのまま保持したもので、生距離同点・width改善。
括弧の種類の置換は行わず、これらを生の完全一致に数えない。
既存のraw正読後の損失は571→546組となり、複数部品や別の接触による損失は残る。

新規3,648画像は本文 `性格` / `枝葉`、同じ3font file、18/30px、palt=0/1、
5closer・先頭/末尾・ASCII空白0/1/2個・PNG/JPEG75・上下紙0/8pxの2,880画像と、
本文だけ/細縦線/太縦線/4×4点の768負例。生成前に組合せと条件を固定した。
font file/glyph/空白advanceを保存し、形状追加前の本文非白画素も保持した。
追加のモデル認識は全ROIを1forwardとし、元Auto用10,944とHorizontal固定384が正常終了した。
Autoで回転する128画像も除外せず、元Autoと有効Horizontalの前処理tensorを全3モデル形状でbit照合した。
30モデルの実入力RGB全3,776件も期待SHAと一致した。

## 製品への接続と再現

純粋画像と公開Sessionの失敗をRedで記録してから、最後のfallbackを追加してGreenにした。
3モデルのSession/Pool/並列batch/単一Session batch/standalone、raw/候補/fragment、
RGBとconfidence bit不変、エラー後再使用、Cascade開始3/2/1を検査した。
製品処理は保存rawの33,735組すべてで、凍結した診断案と一致した。新しい33,735 NN forwardとは区別する。
全選択の旧11・新8画像と、新768負例・Horizontal128画像の915件×3モデルを実APIで再認識し、
本文/raw/confidence bitが固定値と一致した。
全選択の旧11・新8画像と新768負例について、実単行CLI 2,361プロセスも全本文が一致した。
stdoutの最後のLFを厳密に1個だけ除き、空白はstripしない。
ページ287画像の8経路は全件新規実行し、6,888文字/単語指標に退行0。
本文と全行数/矩形/confidenceも不変だった。比較元はsource/input SHAを検証した9fcの保存記録で、新規before推論とは区別する。

通常648件・ONNXスイート570件・example23件が成功し、新規Clippy警告0。
環境変数なしでno-opとなる診断もテスト件数には含まれる。
未知資料での精度や速度は未検証。

`data.zip` は原画像・RGB・truth・renderer出力と実NN入力、`diagnostics.zip` は凍結source/SHA・
生成/認識/採点コマンド・全raw/選択/API結果・Red/Greenを収録する。
`text.zip` は全採点表と実CLI/ページ出力。重複内容はarchive内のaliasesで可逆に復元する。
各JSONがarchiveと全memberのSHAを持つ。モデル・実行binary・font本体は含めない。

```sh
python3 docs/benchmarks/single-pixel-body-bridge/reproduce.py --output tmp/single-pixel-restored
```

画像照合にはPillowが必要。正式scorerで120表ファイルをbyte再現し、API/CLIの固定本文とconfidenceも照合する。
復元先の `product-manifest.json` を `NDLOCR_BRIDGE_PRODUCT_MANIFEST` に設定して、
`verify_product_matches_frozen_bridge_policy` を明示実行すれば、モデルなしで現製品の全33,735組も検査できる。
別ディレクトリで全member/aliasのSHA、120表のbyte一致、3,776実入力RGB、API/CLI値とconfidenceを照合し、
同じ凍結製品binaryで復元後の33,735組も成功した。結果と実行SHAは `verification.json` に保存する。
