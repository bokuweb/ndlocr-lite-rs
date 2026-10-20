# 接触した英字を元画素で支持し、認識済みの丸括弧を保持する

比較元はd711eec。単行のrawが KLM) と正読していても、L/Mが1画素で接触していると、
後処理が文字数を支持できず KLM にしていた。元RGBの所有関係と接触を調べ、
不均等な2つの文字領域を仮のviewに分け、既存fragmentを最後の単行fallbackで保持する。
元RGB・NN入力・モデル・confidenceを変えず、追加NNや単語・文書種別の置換を導入しない。
ページ断片経路と既存H/4幅条件は維持する。

対象は空白なし英字3 scalarと端の丸括弧1個。本文2成分のうちちょうど一方だけが、
左右へ8近傍で接触する1前景画素の列で、固定した幅・高さ・重なりの2 viewに分かれることを要求する。
全前景の所有関係、singleton・間隔・marker自身の画線profileも検査する。
元画素や文字を生成しない。仕様・境界テスト・PBTを先に固定し、追加NN結果へ数値を合わせ直していない。

| 比較 | 条件・モデル・方向の組 | 本文変更 | none完全一致回復 | width完全一致回復 |
| --- | ---: | ---: | ---: | ---: |
| 既存13群 | 162,915 | 18 | 18 | 18 |
| 追加Auto | 58,320 | 94 | 94 | 94 |
| 同じ636元画像の明示Horizontal | 1,908 | 0 | 0 | 0 |
| 合計 | 223,143 | 112 | 112 | 112 |

全群のnone/width/strictで入力別の文字・単語退行0。strictでは括弧が除去されるため指標同点。
112組は元RGB48種類に対応し、追加94組は既存と未重複の42種類である。
モデル・条件・方向・重複を独立標本と数えない。
既知6本文と3font fileによる開発対照で、未知資料・未知fontの改善率ではない。
旧2,871画像×3モデルのraw正読後の完全一致損失は485→467。

追加19,440条件はFreeType BASIC/palt未指定、CoreText/palt0、CoreText/palt1、
3font file/face0、24/40px。KLM/XYZ/MLK/WM/VVM/qjp、本文内空白、
ASCII/全角round closer・前後・端空白・紙0/2/10・PNG/JPEG75を含む。
WM/VVMはW/2つのVの混同も調べる。元RGBは13,368種類で全て旧評価と未重複。
生成前に全tupleを固定し、native描画1,854回と独立1,854再描画で実font/glyph/advance/RGBを照合した。
FTにnumeric glyph IDやpaltを捏造していない。

外側図形9,072条件のbody-only真値は診断上の規約である。
curve/forkは記号にも見え、意味が一意な実資料の非文字GTやholdoutとは扱わない。
4,536 pre-JPEG合成を独立再構成し、全元非白画素の保持と重なり0を確認した。
108例をNN前に目視し、全19,440元Auto/有効Horizontal tensorを3形状でbit照合した。
追加rawは60,228 forward、実入力captureはmodel30の20,076件である。
幾何到達性56条件/50種類は仮rawによる検査で、精度測定ではない。

公開Session/Poolの3モデルRedを先行し、接続後にGreenを確認した。
実製品223,143保存raw再生が凍結候補に全件一致し、公開API9,184組のraw/本文/mean bitも一致した。
全112変更組に加え、事前固定した9,072図形条件へidのSHAで1モデルずつ割り当てた。
CLI9,184件も一致し、stdoutから正確に末尾LF1個だけを除いた。
CLI画像は凍結RGBのlossless PNGで、JPEG75条件もPILで一度decodeした同じRGBを使う。
直接JPEGをRust decoderへ渡す場合の同一性までは主張しない。

通常710件・実モデルCI対象657件・example等24件が成功。Clippyの既存45警告に対し新規0。
実モデルCIには既存ページNN回帰も含むが、新規ページ精度測定は行っていない。
ページ断片/NN経路のsource不変性と、固定policy/kernelの数値不変性を検査した。
モデル/charset/Cargo.lockもraw取得時と一致する。Cargo.lockは計測snapshotとして保存する。

全15 ZIPは50MiB未満で、memberと重複aliasのSHAをindexする。親packetは narrow-round-marker。
モデル・実行binaryの本体は配布せず、実行時source・SHA・全画像/真値/raw/採点表/API/CLIを保持する。
再現処理はモデル不要で親評価も再計算し、追加分1,188 CSV/MDを照合する。
別の空ディレクトリで親評価と全追加データを復元し、1,188 CSV/MDと3監査の一致を確認した。
さらに復元した最終sourceをモデル不要のdefault featureでビルドし、Rustの223,143組再生と
19,440×3形状のtensor bit照合も成功した。終了記録とSHAは verification.json / execution/ に保存する。
重複した生成txtはportable-cases/raw-recordsのUTF-8値から復元し、全本文を保持する。

```sh
python3 docs/benchmarks/latin-pair-bridge/reproduce.py --output tmp/latin-pair-restored
```

原画像・失敗ログを保ったまま、作業容量のため生成済みtxtをbyte同一のCOW cloneへ置き換えた。
native APIで別inode・書き込み分離を先にテストし、全置換の前後bytesを照合した。
Node22の強制clone APIは私的testでENOSYSとなり実データには使わなかった。
この容量調整はportable再現の依存ではない。
単一pixels ZIPがサイズ制限を超えた初回も保存し、入力を変えずfont別とatlasへ分割した。

Latin以外、長い本文、複数closer、真縦組みを含むAuto方向などの改善を継続する。

初回CIの標準3ジョブは cargo fmt で失敗した。評価後の修正はテストコード2ファイルの
rustfmt整形だけで、実行時Rust全体と数値条件は評価時とbyte同一である。
execution/final-format/ に前後source・比較監査・失敗/成功ログを保存する。
整形後の純粋25件と3モデルの公開API回帰（1テスト/9呼出し）も成功した。
ZIP内sourceは測定時の版で、最終整形版との対応は同監査で確認できる。

整形修正コミット e42eb19 のCI39374618726は標準3OSと実モデル2OSの全5ジョブが成功した。
初回9b60702も実モデル2OSは成功し、標準3OSの失敗は整形だけだった。
両実行の終了記録と初回失敗ログを execution/ に保存する。
