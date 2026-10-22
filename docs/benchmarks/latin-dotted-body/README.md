# 点付き英字の画線所有を調べ、認識済みの丸括弧を保持する

比較元はf31a87d。rawが`)Agjp`と正読しても、jの点を別文字に数えるため単行後処理が
`Agjp`へ落としていた。小さな上部部品を一意なstemへ所属させ、文字位置・全前景・本文整列と
丸括弧の画線profileを確認し、最後の単行fallbackで元fragmentを保持する。
空白なしASCII英字3〜8字と端のround closer1個、点付きi/jは1〜2字を対象とする。
単語・font・文書種別の照合や文字置換を行わず、元RGB/NN入力/raw/meanを保つ。追加NNはない。

| 比較 | 条件・モデル・方向の組 | 本文変更 | none完全一致回復 | width完全一致回復 |
| --- | ---: | ---: | ---: | ---: |
| 既存15群 | 223,143 | 12 | 11 | 11 |
| 追加Auto | 58,320 | 64 | 22 | 54 |
| 同じ334元画像の明示Horizontal | 1,002 | 26 | 12 | 26 |
| 合計 | 282,465 | 102 | 45 | 91 |

全入力・全none/width/strictの文字・単語退行0。none文字距離573,245→573,189、
width文字距離553,121→553,019。strictは指標同点。102組すべてが完全一致になるわけではなく、
既存1組などはi/jの誤読を残したまま括弧だけを回復する。旧2,871画像×3モデルのraw正読後の
完全一致損失は467→456。別baselineの過去段階の改善数とは合算しない。
102組は23種類の元RGB、追加分は旧評価と未重複の19種類に対応する。
モデル・方向・重複RGBを独立標本として数えない。

追加19,440条件は描画/NN前に固定した。FT BASIC/palt未指定、CT palt0/1、3font file/face0、
20/32px、Abjp/Ajbp/Bpjg/Adip/Aimq/Apji、本文内/外側空白、round2種、前後、紙0/2/10、
PNG/JPEG75、9種の外側図形を含む。元RGB13,320種類は全て旧評価と未重複で、真値競合0。
適用は今回の丸字体の開発対照に限られ、未知書体/資料のholdout改善率ではない。
curve/fork等のbody-only真値は診断上の規約であり、意味が一意な実資料の非文字GTではない。
図形9,072条件×3モデルに本文変更はなかった。

owner・policy・numeric boundsを追加NN前に凍結した。純粋30テストと4つの64例PBT、
実画像、実Session/Poolの3モデルRed→Greenを先行した。初期のmarker実装は実曲線を拒否し、
中心以外にも最大x接触を要求していた点を、別の小さいRedと仕様の明確化を経てNN前に修正した。
追加NN結果へ条件を合わせ直していない。接続時はpolicy先頭doc commentだけを更新し、
owner SHAとpolicy bodyは凍結値と同一。readerのClippy修正とtest-only整形の同値証明も残す。

native描画1,854回と独立1,854再描画で実font/glyph/advance/RGBを照合した。
独立BFSと実Rustで裸本文126条件の全成分を照合し、最大成分高を図形の基準とした。
4,536 pre-JPEG図形を独立再構成し、微小なAAを含む元非白画素の保持と重なり0を確認した。
108条件をNN前に目視した。全19,440元Auto/effective Horizontal tensorを3形状でbit照合し、
model30の実NN入力19,774件も全ROI/ID/寸法/RGBを照合した。新rawは59,322 forward、失敗0。

実製品の保存raw全282,465組が固定候補と一致した。実API426組（変更102＋事前固定noise324）の
raw/本文/f32 mean bit/RGBも一致し、本検証852 forwardと種テストのRed/Greenは別に記録した。
単行CLI426プロセスも保存期待値と一致し、末尾LFを正確に1個だけ除いた。
CLIにHorizontal指定がないため、APIのHorizontal26組には同じ元画像のAuto26組を対応させ、
それぞれ別の方向の期待値を使った。CLI入力は凍結RGBのlossless PNGで、JPEG直接decodeの
同一性は主張しない。詳細は `docs/latin_dotted_cli_direction_note.md`。

通常740件、実モデルCI対象692件、example等24件が成功。fmt/diffとClippyも成功、新規警告0。
実モデル回帰は既存ページNNを実行する。今回の追加ページ精度測定は0で、NN/ページ断片経路の
sourceは比較元とbyte同一である。

全ZIPは50MiB未満で、member/aliasのSHAをindexする。モデルと実行binaryは配布しない。
全282,465組に必要な元RGB/raw/真値/比較前後本文、153 CSV/153 Markdown、確率、API/CLI、
全追加画像、source/失敗履歴を自包する。親Cの全履歴を再実行したという主張はせず、
親packetと過去の成功receiptをSHAで固定し、今回の全採点表を独立に再計算する。

Pillowを使えるPythonでリポジトリルートから実行する。復元先は空を要求する。

```sh
python3 docs/benchmarks/latin-dotted-body/reproduce.py --output tmp/dotted-reproduced
```

復元後の `product-manifest.json` は絶対RGBパスを持ち、モデル不要の実Rust再生に利用できる。
復元sourceの `infer::visible_repetitions::dotted_body::tests::product_matches_all_frozen_outputs`
を `NDLOCR_DOTTED_PRODUCT_MANIFEST` と `--exact --ignored` で実行する。
実行・独立復元の成否と最新CIは、外側の `verification.json` と `execution/` に追記する。

独立復元で306採点ファイルと711群集計が一致し、復元sourceの新規default buildでも
Rust282,465組再生と19,440×3形状tensor bit照合が成功した。
