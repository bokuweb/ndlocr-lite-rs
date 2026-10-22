# PR21引継ぎの精度改善: 採用結果と残差レビュー

現データと固定モデルで根拠を得た改善を本体へ反映し、用意した追加候補の採否を閉じた。
未評価の候補を採用している箇所はない。残る誤りが0になった、あるいは将来の改善が
不可能という意味ではない。既知fontの合成対照から未知資料の改善率を推定しない。

## 採用済みの最新状態

Hは画像支持付きの短い縦列回復、Oは容量上限での完全prefix延長、FVBはPageの
未検出単縦列、FVC v2はCLIの同じ構造的欠落を扱う。FVC v2は既存initial結果を再利用し、
quality boostなど既存finishを一度だけ通す。文書種固有の文字置換はcoreへ追加していない。

- [FVB](vertical_page_fallback_integration.md): 921 NN入力/922採点ID、44,256 CSV行・
  22,128比較で入力別文字/語悪化0。新本文raw7経路組改善、CLI3,684本文不変。
- [FVC v2新72](vertical_cli_fallback_v2_results.md): 576 CLI結果/1,728 CSV行、
  10本文画像/raw22採点経路組改善、悪化0、非本文36枚不変。
- [FVC v2全件](vertical_cli_fallback_v2_full_results.md): 1,311物理入力/1,317採点ID、
  26,220結果/79,020 CSV行/39,510比較、全59 jobs成功。悪化0、Page本文/bbox/f32 bits変更0。
  raw820 gain組は既知scopeで、新規820画像ではない。
- [Root接続](vertical_cli_fallback_v2_root_results.md)と[本体CI](vertical_cli_fallback_v2_root_ci.md):
  CLI2 Red→2 Green、quality3設定一致、ONNX lib468・通常lib456・parser10成功、
  feature無効16 process不変。全5 CI jobs成功。モデルasset timeoutの初回失敗も保持した。

横書き・DEIM領域・長行・literal記号・字形部品の先行変更は[計画](plan.md)と各仕様に記録する。
上記件数へ先行phaseの改善を合算して全体の改善率を作らない。

## 今回閉じた候補と、その根拠

| 課題・候補 | 判定と根拠 |
| --- | --- |
| 容量末尾の一文字訂正N | [414入力で新規の訂正改善0](vertical_uncertain_scalar_validation.md)。完全prefixだけのOを採用し、N全体は保留 |
| 途中欠落への挿入Q | [新72の全比較で新規改善0](vertical_unique_internal_results.md)。既知例だけで採用しない |
| 低信頼停止S | [新72の3,456結果が全本文/mean bits不変](vertical_low_stop_results.md)。閾値を合わせ直さず保留 |
| 中程度長の完全prefix R | [新72の3,456結果/10,368 CSV行で新本文改善0](vertical_moderate_prefix_results.md)。全1,728組の本文/mean bits不変、本体未接続 |
| 単列モデル合意C1 | [新96の768結果/2,304 CSV行で新規改善0](vertical_cli_model_agreement_results.md)。全C1保留 |
| 50/100の合意をさらに追加 | [実資料6列では両モデルとCLI DEIMが既にbyte一致](vertical_scan_direction_diagnostic.md)。この残差の新しい選択器には改善根拠なし |
| raw保持条件の緩和 | [実primary二分割117 raw/297 CSV行](real_vertical_primary_partition_results.md)で全文合意があっても完全raw prefix延長0。空白・字体をGTへ合わせて合意を作らない |
| EOSを低信頼停止とみなす | [実停止18入力](real_vertical_additional_stop_probe_results.md)はEOS17/低信頼1。観測と異なる停止理由を補正条件にしない |
| 画像支持のないFVA | [新規対照の悪化](vertical_page_fallback_results.md)を保持し、出力支持付きFVBへ分離済み |
| 灰色紙のOtsu方向maskだけを追加 | [追加NN0の画素診断](vertical_scan_direction_diagnostic.md)で地獄変のanchorが0/3/1。既存支持6に届かず単独案を進めない |
| confidence/余白合意だけで空白を削る | [比例組版の反例](punctuation_space_rejection.md)があり既定補正を撤去済み。confidence上昇を正読の代用にしない |
| 文書に特化した誤字・隣接行の置換 | AGENTSの範囲どおり外部`--post-dict`/`--rule-pack`。汎用coreへ既知本文の正解を内蔵しない |

これらの保留案を同じ対照へ合わせて再調整する未完了作業は残さない。
Rの完了readerは未実装Red→2 Green、R関連のモデル不要12 testsが成功した。
最後の背景診断は4 Red→4 Greenで、元画像/GT/モデル/本体runtimeを変更していない。

## 残る誤りと評価の限界

実資料の空白・字体・少数誤字、30モデルの途中欠落、背景が暗い列のbands誤方向は残る。
単に全入力を大きいモデルへ変える、同じGTで支持閾値を下げる、合意が取れない文字を置換する
ことは、ここまでの無回帰・新規対照の根拠から採用できない。

次に別の精度改善を主張するには、紙面方向/字形の独立注釈を備えた追加資料、未知fontを
含む未測定対照、または重み/decoder/モデル変更を対象にした別の評価仮説が必要になる。
これらは未実施の評価・研究であり、改善が保証された残作業ではない。
今回列挙・事前固定した具体案は上表まですべて評価済みで、PR22には検証済みの本体だけを残す。
