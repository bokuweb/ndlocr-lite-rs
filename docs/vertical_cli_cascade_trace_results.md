# 既知3入力のCLIモデル選択trace結果

Linux run39552644890（e45db7a）は成功した。通常O/1f240beと観測版をfresh releaseでビルドし、
3画像×default/単一30/単一50×2 sourceの18 CLI processを実測した。
すべての通常/観測stdout・stderr・bodyが一致し、27回分の記録crop RGBは元PNGの矩形と一致した。
9回の非空pool batchを観測したが、内部forward/session数は未計測null。
none/width/strictの全54 CSV行は独立距離と一致し、観測版が追加NNを要求することはなかった。

| 既知入力 | default / 最初の100 | 単一30 | 単一50 |
|---|---:|---:|---:|
| advance_bold_71_pad8 | 24 scalar | 71 | 71 |
| composite_arial_67_pad0 | 22 | 3 | 64 |
| advance_serif_83_pad0 | 2 | 1 | 83 |

3画像のdefaultはinitial100だけが非空で、initial30/50・retry50/100はすべて空だった。
initial-finalとpage-batch-finalも本文/confidence bitsが不変だった。
したがってこの3例の欠落を、promotionで長い結果が上書きされたものとは説明しない。

boldの30/50は同じ全文71 scalarで、100の24 scalarをそのままprefixに持ち、confidenceは両方0.99以上だった。
しかしserifの30は「(」、compositeの30は「の中國」で、50だけの有効そうな出力とは一致しない。
compositeの50には末尾の誤字/欠落も残る。30/50への一律切替や、高confidence一つだけの採用根拠にはしない。
これらは既知の原因調査例であり、新規対照での改善数ではない。本体のモデル順序・閾値は変更しなかった。

最初のattempt39552273440はcompile-time PNGのpacket同梱漏れで純粋testコンパイルが停止し、CLI認識0。
元bundle/sourceと失敗artifactを保存し、同じ1f240beのfixtureを加える回帰をRed→Greenにした。
推論Rust・モデル・3入力は修正前後で不変だった。

モデル不要readerは公開packetの全member、実build/argv、通常/trace本文、27 crop、全54指標行と
各stageの結果を照合し、pool batchと内部forwardを区別する。今後のモデル選択案は別仕様と新対照で検証する。
