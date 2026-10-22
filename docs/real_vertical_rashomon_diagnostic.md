# 羅生門の列内欠落の診断（追加計測前仕様）

方向fallback FVBの先行Page比較で、PID1088339/コマ6/印刷1頁の3列はOとFVBの全18物理入力/経路組で
本文・矩形・confidenceが一致した。FVBを採用済みとは扱わず、この診断でも製品処理を変えない。

既存30モデルのeval/defaultは、各列の本文が30/21/25 scalarになっていた。
GT本体は34/33/34 scalarであり、列の途中にも欠落がある。50/100では本体がより保持される一方、
余分な空白もある。長さだけで出力上限や幾何欠損が原因だと断定しない。
既存Oは元rawの完全prefixだけを延長し、任意の途中挿入や未飽和出力は回復しない設計である。

## 対象と観測

- `docs/real_vertical_rashomon_control.md`でNN前に固定した3物理PNGをそのまま使う。
  GT二表記は評価だけに使い、画像・NN呼数・資料数を増やさない。
- 計測元はO/1f240beと同じ推論source。別コピーへcfg(test)の観測だけを加える。
  既存P1記録器を再利用できる場合も、144件固定の旧collectorを新しい3件の実行に偽装しない。
- Page model30/50/100×eval/defaultの18 API結果を保存し、各caseのDEIM採用枠、
  実primary RGB/寸法/向き、raw復号直後、H/O後・sanitize後、最終Pageの順に結ぶ。
- 選択outputの静的T/C、実T一致、raw scalar数と停止情報、Hの対象可否と採否、
  O plansの成立/不成立、成立時の両分割境界・元RGB部分画像・全raw部分返値・combine採否を記録する。
  記録していない停止理由や内部状態は推測で埋めずnullとする。
- 実session.run試行/成功/失敗を記録し、Page API回数やモデルloadと区別する。
  元RGB cropを独立に検算する。Ruby画素は保持されるため、主本文の字形BBoxが無い箇所を
  「glyph欠損0」と断定しない。
- 同じPNG/モデル/設定の既存Oの18結果と、本文/順序/矩形/confidence bitsを全件照合する。
  観測による出力変更があれば診断失敗であり、精度改善とは数えない。

記録器の順序/バイト保持・無効時no-op・drain後無効化の既存Red/Greenを保つ。
新collectorは入力件数・必須環境・欠落結果を検査し、未実行を成功にしない。
source/モデル/charset/入力/collector/実行物をSHAで固定し、freshな対象artifactを必要とする。
実行資源に余裕がある環境で1 NN processずつ実行し、中断/partialは完了へ数えない。

この既知の1頁/3列は原因診断であり、新規holdout改善ではない。途中欠落を直す政策はまだ固定しない。
観測後に候補を作る場合は、別仕様・先行Red/Green・未測定対照・既知全件の非悪化を要求する。
