# S先行結果: 新native対照の改善0で保留

`docs/vertical_low_stop_recovery.md`を実装・新対照・追加NN前に固定した。
隔離コピーだけへ実装し、本体の推論はO/1f240beから変更していない。

停止metadataは既存デコーダobserverを一度だけ使う。EOS/LowConfidence/全step完了、エラー、
停止後logit、shape tailと128例propertyを3 Red→4 Greenで確認した。純粋採否、
一意内部挿入（各512例の2 property）、quarter分割（64例property）、
元RGB再認識と途中失敗の保持を小さくRed/Greenし、計24試験が成功した。
最初のquarter buildは容量不足でテスト実行前に失敗しており、Redへ数えていない。

旧geometryの共有化前後は486 RGB×3modelの1,458組で準備RGB・境界がbyte一致した。
細分化は全558 unique RGB・注釈378画像×3modelの1,674条件を保存し、候補1,141条件の
divisor2 crop8,058個/divisor4 crop16,089個が独立PIL回転/切出しと全画素一致した。
glyph内部切断0、NN/load0。ハイフンを含む既存IDを記録器が拒否した最初の失敗も保存した。
これは幾何成功であり、実際のH停止条件を満たす画像数ではない。

既知Q serif50/pad14/model30で、未接続Red1件→接続Green1件を実モデルで実行した。
H結果43文字を残して7文字を挿入し、50文字となった。mean bitsは1065186501から1065149311へ
下がり、5対象外経路は元本文/mean bitsを保持した。単件/batch/専有batchと、不正入力後の再利用も一致した。
この既知例の成功と、新規対照の改善を区別する。ONNX非ignoredは462成功、115 ignoredで、
別途強制したモデル回帰1件はignoredのまま未実行だった試験として数えていない。

新72（本文54/図形18）は事前に固定した既知3fontの合成開発対照で、3,870 glyphを独立配置した。
前後6コマンドはすべてexit0、API error0、source/入力/model/charset不変。
3,456画像結果、2,604外側API、18 model session loadで、内部forwardは未計測null。
元の凍結O baseline実行物を再利用したが、72画像のbaseline予測はすべて新たに測定した。

全1,728前後入力/経路組は本文/mean bitsが一致した。5対象外経路の変更0、
各variant内のFragment単件/batch/専有batchは全件一致した。
公式none/width/strictの10,368 CSV行を独立距離・分母と照合し、文字/語比較10,368件は
悪化0・改善0だった。

判定は **HOLD_S_NO_NEW_NATIVE_GAIN**。全既知558 nativeや全Page/CLIの採用比較へは進めず、
source・全観測・失敗を保存する。Q/Sはいずれも本体へ未接続であり、FVBの別gateと合算しない。
