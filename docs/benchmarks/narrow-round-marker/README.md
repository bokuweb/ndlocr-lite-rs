# 狭い丸括弧の元画素profileによる支持

比較元は1ec730a。既存のH/4幅条件に届かない丸括弧に対し、元RGBの行ごとの連続画線、
左側の上下端、中央を含む右側の帯を検査する。本文は既存の1画素接触と大きな分離部品の
所有関係を共有する。単行の最後のfallbackだけでモデルが既に読んだfragmentを保持し、
単語・文書種別の置換や追加NNを導入しない。strict経路の幅条件は維持する。

| 比較 | 条件・モデル・方向の組 | 本文変更 | none完全一致回復 | width完全一致回復 |
|---|---:|---:|---:|---:|
| 旧11群 | 137,991 | 45 | 45 | 45 |
| 追加Auto入力 | 24,192 | 11 | 11 | 11 |
| 同じ244元画像の明示Horizontal | 732 | 6 | 6 | 6 |
| 合計 | 162,915 | 62 | 62 | 62 |

全群のnone/width/strictで各入力の文字・単語退行0。strictでは変更した括弧が除去されるため指標同点。
62組は19種類の元RGBに対応し、モデル・条件・方向・重複を独立標本として数えない。
追加で変わる17組は未重複の元RGB6種類である。既知の2文字列・3font fileによる開発対照であり、
未知資料や未知fontに対する改善率ではない。旧2,871画像×3モデルでraw正読後に完全一致を失う組は497→485。

追加8,064条件はFreeType BASIC/palt未指定、CoreText/palt0、CoreText/palt1の3描画modeで、
同じ3font file、20/28px、本文2種・本文内空白・ASCII/全角round closer・前後・端空白・紙0/2/10・
PNG/JPEG75を含む。文字3,456条件と8種の外側図形4,608条件を全て保持した。
元RGBは5,568種類で、そのうち5,304種類が旧137,991組と未重複である。
条件行は7,776未重複/288重複で、同じRGBの異なる真値は0。追加native描画648回と、
別の648再描画検査で実font/face/glyph/advanceとRGBを照合した。
FreeTypeでnumeric glyph IDやpalt軸を作ったとは主張しない。

全非白画素を2px白紙で囲うROIを使い、元nativeの内部advance・空白・文字画素を保つ。
2,304 pre-JPEG図形組を独立再構成し、全元文字画素保持と重なり0を確認した。
curve/forkなどの図形は記号に似ている。本文だけの真値は診断上の規約であり、
実資料で意味が一意な非文字ラベルやholdoutとは扱わない。これらも結果で除外していない。

数値条件をNN前に凍結し、全8,064元Auto/有効Horizontal tensorを3形状でbit照合した。
幾何到達性の4未重複例は仮rawによる確認で、精度測定には数えない。
追加raw評価は24,924 forwardで、実入力8,308件を照合した。
旧新・3描画mode・文字/図形・重複/未重複・元Horizontalの全群で非悪化を要求した。
閾値を測定結果へ合わせ直していない。

公開接続前に純粋テストと実SessionのRedを固定した。3モデルのrawは `)情報` だが、
処理後が全て `情報` になることを確認し、最小接続後にGreenにした。
製品の全162,915保存raw再生は凍結候補と一致した。

公開API8,564組のraw/最終本文/f32 mean bitと元RGB、実CLI3,500件の本文が全件一致した。
代表例では全3モデルのSession/Pool/Batch/Cascade/fragment/候補経路を確認した。
通常685件（25 ignored）・ONNX CI対象627件（0 ignored）・example23件が成功した。

Clippyでテスト用RGB変換の反復方法に新規警告が出たため、既存テスト内の2箇所だけ
chunks_exact_mutからchunks_mutへ変更し、境界テストとClippyを再検証した。新規警告0。
計測したnative/CLI snapshotと最終sourceの差はこの2箇所だけで、全runtime sourceはbyte同一である。
修正前の検証結果と警告、最終source/コンパイルの記録を両方保存する。

新規ページ精度測定は行っていない。既存native回帰テストのページ経路検証とは区別する。
app/PageRecognizer/fragmentのsource不変性を照合し、単行の支持条件だけを変更した。
Latinのx-height、長い本文、複数closer、真縦組みなどの残差施策は残る。

## 保存と復元

images/pixels/inputsに全追加媒体・元RGB・有効RGB・実入力・真値・native記録、
diagnosticsに凍結source/仕様/raw候補/全記録/失敗を含むログ、textに全正式表とCLI出力を格納する。
各zipは50MiB未満で、SHA/member SHA/aliasを同名jsonで固定する。比較元owned-parts-body-bridgeと
その親packetをSHAで継承し、モデルとOS font fileは配布しない。

PillowのあるPythonで次を実行する。多数の小ファイルを作るため、作業用空き容量は16GiB程度を確保する。

```sh
python3 docs/benchmarks/narrow-round-marker/reproduce.py --output /absolute/new/output
```

portable再現は保存画像/traceの検査と正式scorerの再計算であり、NNやnative再描画は行わない。
CoreTextのpalt/文字/size、font path/run、glyph/index被覆、FTのBASIC/face0/実glyph mask記録を検査する。
元環境では保存したverify_native.pyが全648 nativeを実際に別描画してRGBまで照合した。
親と全162,915組のraw/真値/比較元本文、全追加画素・元文字保持・実入力、実API/CLI/meanを照合する。
旧/新/HのCSV・Markdown450ファイルと3 auditを再計算し、byte一致と各表の文字/単語退行0を要求する。
復元した元RGBを使う凍結Rust製品再生とtensor照合も別に記録する。

独立復元は終了コード0。全450表ファイル・3 audit、全162,915組の親比較、全画素/実入力/trace/mean/API/CLI/sourceの検査を通過した。
復元した元RGBに対する凍結Rust製品162,915組と、8,064×3形状tensorのbit照合も成功した。
詳細なコマンド・source/binary/入力/出力のSHAと終了コードはverification.jsonに保存する。
