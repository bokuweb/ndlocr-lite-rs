# Q先行結果: 新native対照の改善0で保留

Qは`docs/vertical_unique_internal_insertion.md`の固定方針を隔離sourceで実装した。
元rawの全文字/空白を順序どおり残し、二つの分割合意から一意な内部連続部分だけを挿入する。
既存O/Hの採用を優先し、元のO partsだけを使うためQ自身の追加NN/loadは無い。

- 最初のcompile失敗（property assertionのformat文字列）を残し、rootと同じproptest1.12.0へ揃えた。
  helperは3 Red、構成的property追加後5 Red→9 Green、支持/meanは3 Red→5 Green。
  2 propertyは各512例で、全位置を総当たりするoracleと元UTF-8保持を検査した。
- 既知P1/P5の保存O parts105組はRust/別Pythonの採否・mean bitsが全件一致した。
  counterfactual採用は羅生門1列・model30の2設定のみ。12 CSV行を独立距離で照合した。
- 実Pageは隔離O Red1件/Q Green1件。GTに無い元ASCII空白一字を保持したまま、
  生文字距離6→1/語2→2となり、矩形/confidence bits・eval/default・不正入力後再利用を保持した。
  これは既知1資料/1頁の開発結果であり、独立未知資料の改善率ではない。
- 新72を追加NN前に固定した（既知3font、size41/advance49、54本文/18図形）。
  3,348 glyphのdraw/getmask配置、全PNG/RGBが一致し、新旧の重複0。
- 旧198+新72の6,480幾何記録/26,506 cropは独立PILと一致した。
  supported glyph内部切断0/legacy156、既知198の4,752記録は公開canonicalと一致した。
  幾何成功を精度改善として扱わない。
- 隔離Qの通常lib426成功/38ignored、ONNX nonignored438成功/112ignored。
  モデル付きPageの実行1件はこのignored件数に含めず、別途強制実行した結果である。

## 新72 native先行比較

`docs/vertical_unique_internal_native_preflight.md`に従い、freshで異なる前後実行物を凍結し、
全3model/8経路を6コマンド直列で測定した。全コマンドexit0、error0、source/input/model不変。
3,456画像結果/2,604外側API/18 model session loadで、内部forwardは未計測nullである。

全1,728前後入力/経路組の本文とmean bitsが一致し、Fragment単件/batch/専有batchも一致した。
5つの対象外経路の変更0。公式none/width/strict全10,368 CSV行の誤りと分母を独立実装で検算し、
10,368文字/語比較は悪化0・改善0だった。

判定は **HOLD_Q_NO_NEW_NATIVE_GAIN**。既知羅生門の成功だけで採用しない。
Qの全既知414/nativeやPage/CLIの高価な採用比較へ進めず、source・反例・全結果を保持する。
本体推論はO/1f240beのまま。方向fallback FVBの別の全件CI結果とも合算しない。
