# 追加実資料6列の実Page trace結果

事前固定した6列を3入力ずつ二つのbatchで観測した。P5/Oの既存fresh実行物を再利用し、
モデル30/50/100×eval/defaultの全36 Page結果がFVB本体e559 baselineの本文・行数・bbox・f32 confidence bitsと一致した。
全primaryはusing_bands=falseで、OとFVBの未検出fallback差は通っていない。
成功PageRecognizer loadは6、対応する明示Page model session loadは12。
実PARSeq forwardは66試行/66成功で、未観測baselineとDEIMの内部forward数は不明のまま。

元画像bboxとのprimary36 RGB、独立回転/切出しとのO part30 RGBが一致した。
全174観測RGBを保存し、before/trace×6経路×11採点表記×3正規化の396 CSV行は独立距離と一致した。
source/lock/実行物/モデル/charset/入力/GT/driver/既存結果の認識前後SHAも一致した。
この観測は透明性と原因の切り分けであり、新しい精度改善ではない。

一葉3列のmodel30はraw段階ですでに途中/末尾が脱落し、H採用はなく、O planも成立しなかった。
raw scalar長は各23/22/11（末尾空白を含む）で、frame limit31の容量付近条件30..31に届かない。
最終本文の欠損を後処理が生んだとする根拠はなく、col3では末尾空白だけがsanitizerで除かれた。
P5は実decoderの停止理由やHの個別partを記録していないため、EOS/低confidenceなどの停止原因は断定しない。
既存QはO planを、SはH実採用とLowConfidenceを必要とするので、この3列を救う適格性はない。

地獄変のmodel30は3列ともO planが成立したが採用はなかった。
col1は両分割が同じ本文でも空白列とraw prefixを変え、col2は分割間の空白不一致、
col3は両分割が同じ全文でも元rawの「土」を「王」へ変えるためraw prefixを保持しなかった。
このGTを見た文字置換や空白削除は追加しない。50/100は全6列でO容量条件に届かず、Hも採用されなかった。

公開packetは`docs/benchmarks/real-vertical-additional-primary-trace/`。
readerは未実装Red1→Green1で、既存公開controls/baselineも認証し、全36対応・174 RGB・396 CSV行・O棄却理由を再検算する。
モデルやネイティブ実行物は不要。出力ZIPはmember単位の内容address transportで、元containerのbyte再現とは区別する。
本体推論とQ/S/C1の保留状態は変えず、FVCの全件gateも別に継続している。
