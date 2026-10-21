# 追加実資料6列のFVB baseline診断

FVBを本体へ接続したe559efbの全5 CI jobs成功後、事前固定済みの二冊・6物理列・11採点表記を測定した。
新しい改善案の前後比較ではない。実行物は本体sourceからfreshにビルドし、別pathへコピーしてSHAを固定した。
3モデルのPage eval/defaultで36 API結果、CLI DEIM/bandsで12 process、計48画像結果を保存した。
公式none/width/strictの264 CSV行は独立距離計算の誤り数・分母と一致し、source/モデル/入力/GTのSHAは測定前後で一致した。
PageRecognizerの成功loadは3、対応するPageモデルsessionは6。CLI session/内部forward数は未計測null。

| 経路 | noneの文字誤り / 405 | noneの語誤り / 11 |
|---|---:|---:|
| Page30 eval/default（それぞれ） | 178 | 12 |
| Page50 eval/default（それぞれ） | 32 | 18 |
| Page100 eval/default（それぞれ） | 32 | 18 |
| CLI DEIM | 32 | 18 |
| CLI bands | 457 | 160 |

分母は11表記を別々に採点した合計で、11の独立した物理画像を意味しない。
同一担当による二度目視GTであり、専門家注釈・学習未重複とは主張しない。
原画像・ruby・本文GT・曖昧表記の扱いは認識前のprotocolを保持した。

Page50/100とCLI DEIMの本文は6列すべてで一致する。Page50/100の矩形も同じだが、confidenceは異なる。
Page30の一葉3列には大きな途中脱落が残る。今回のPage出力だけでは内部raw/H/OやCLIのモデル昇格過程は分からない。
空白挿入、字体差、少数の誤字も残り、例えば「紺絣」は「馴染」と認識された。文書固有の置換はcoreへ加えない。
CLI bandsの構造的な失敗は、別に事前固定したFVCの検証で調べる。

モデル不要の追加診断は88採点組すべてで生距離が公式noneと一致した。
NFCだけを両側に適用した別欄では、Page50/100・CLI DEIMの文字誤りが32→28となる。
差4は一葉第3列の「突」U+FA55→「突」を4採点aliasで重複計上したもの。
製品出力・GT・公式CSVは変更せず、これを採用済みの精度改善へ数えない。
吳/呉、徳/德、全半角をNFCで同一視しないことも試験で固定した。
