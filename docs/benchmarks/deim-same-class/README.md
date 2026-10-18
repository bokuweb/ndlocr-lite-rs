# 同クラスの近似同一領域の重複抑制

先行仕様 `../../deim_same_class_duplicates.md`。
既存90% IoU選択を同クラスにも適用し、最高のDEIM検出スコアを一つだけ認識する。
本文一致では選ばず、生DEIMと勝者metadataは変えない。

80文字以上の再分割は旧中央二分のままビルドした独立binaryで比較した。
固定46画像（新26+既存20）×DEIM/bandsの全276文字指標が非悪化。
新規対照のDEIM4組だけ改善し、他88本文はbyte同一。
既存20画像の40本文はすべて同一。

| 新規対照・既定DEIM | 生文字誤り before→after |
| --- | ---: |
| observations_gothic24 | 116→18 |
| uri_gothic24 | 244→105 |
| uri_gothic36 | 242→17 |
| uri_mincho24 | 129→7 |

新26画像の前版52 CLIと候補92 CLIを実行。既存20画像の比較元40本文は
同じbinary SHA・画像SHAを確認した `../url-hyphen/*/after/` から凍結再利用。
全前後本文・真値・採点は保存し、この再利用を新規の実行とは数えない。
argv/状態はbaseline-runs/after-runs、binary/model/画像/真値SHAは各manifest。

単体2 Red→3 Green、実DEIMを含む4件、通常507件が成功。
fmt/diff/対象Clippy成功（既存17警告）。model-initial-assertion.logは、
PARSeqの認識confidenceとDEIMの検出スコアを混同して勝者bboxを誤期待した失敗。
訂正後はDEIM全スコアをraw-detections.jsonへ記録し、勝者metadataとの一致を確認した。

長行の切断位置変更はこの比較に含めない。既知/合成対照による評価で、
未知の実スキャンへの汎化や全URLの正読は未確認。
