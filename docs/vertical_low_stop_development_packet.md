# P5 / Q / R0 / S0 / Sの開発証拠を保存する仕様

本体へ未採用の案と採用済みOを混同しないため、各案の先行仕様、失敗、source snapshot、
新対照、実測結果、独立採点を`docs/benchmarks/vertical-low-stop-development/`へ保存する。
Qは新規native改善0で保留。R0/S0は既知例の診断で、未知資料の精度測定ではない。
Sの先行比較が未完了なら未完了と明記し、部分結果を全件成功へ数えない。

既存O/FVBの公開packetを置き換えない。新しいarchiveにはモデル・font・native実行物を含めず、
公開sourceと固定assetsから再buildする。元の実行物SHA、fresh artifact記録、source/入力の
SHA結び付きは維持する。rootのCargo.lockは引き続きignoredとする。

各論理ファイルへSHA256/サイズと内容アドレスpayloadを結び付ける。共通内容は一度だけ保存し、
全payloadとarchiveのSHA/サイズ、memberの一意性、余剰/欠落を保存後に実byteで検算する。
大きい幾何crop archiveは元archiveのSHA/サイズと全memberのSHA/サイズを保存し、
元PNG/RGB・境界・collector sourceから再構成できる形にする。元ZIPの圧縮byteの再現と
crop RGB byteの再構成を区別する。原archiveはprivateに保持し、公開保存のために削除しない。

NNなしの公開検算では全member結び付き、新対照PNG/RGB、保存境界とglyph内部切断、
cropの独立回転/切出しSHA、保存native全経路の一致条件と公式CSV全行の誤り/分母を検査する。
正規化はnone/width/strictを公式仕様どおりに別実装し、既存公開fast distanceを距離計算へ使う。
これは保存済みNN観測値の検算であり、別hostの実行物を再実行したと主張しない。
