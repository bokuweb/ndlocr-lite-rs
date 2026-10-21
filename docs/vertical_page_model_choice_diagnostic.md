# 保存Page結果によるモデル選択差の診断（追加NN・新方針前）

FVBのLinux Page30/50/100が完了し、モデルごとの出力差を追加NNなしで調べられる。
CLIのcached cascadeは認識長が閾値以上なら上位モデルへ再認識するが、Pageは単一モデルの
別callgraphである。保存Page比較を実CLI cascadeのtraceや採用条件と混同しない。

- 全Page job・分割coverage・source/build/argv・認識前planを既存readerで検証する。
  モデル/charsetは凍結bundleのSHAと照合する。固定済み保存metadataのraw上限を使う。
- 30/50の1行出力が縦長（height>=8*width,width>=12）、confidence finite0.9..1で、
  保存metadataのraw上限を超える場合だけ調べる。これは回復のproxyであり、H/O採用の直接traceではない。
- 同じ入力・設定の100モデルと比較し、認識前に固定済みの全本文GT表記（空GT対照は対象外）で生文字改善かつ語非悪化の組を記録する。
  GTは診断・採点のためだけに使い、認識選択条件には渡さない。
- 単一の同じbboxか、異なる/欠けたbboxかを分ける。同じbboxなら100の短縮・他の文字差と、
  100本文が小モデル本文の完全prefixかも記録する。bbox不一致を同一cropの比較と呼ばない。
- 純粋分類の正例、上限/縦横比/confidence、全GT表記、bbox不一致を先にテストする。
  出力だけからcascadeが実際に結果を捨てたと断定せず、追加NN/新policy/本体変更は0とする。

根拠が得られた場合も、後続の実CLI trace・別仕様・Red/Green・未測定対照と全件gateを必要とする。
FVC既定モデル/モデル順序/閾値を、この診断を理由に途中変更しない。

保存分類のreplayは`tools/vertical_page_model_choice.py --artifacts DIR --output NEW_DIR`を使う。
DIR内の`remote-page30-full-v1`/`remote-page50-full-v1`/`remote-page100-shard0..3-v1`だけを
Page診断の入力として検査する。全6 job、全3モデル・前後・eval/defaultのcoverageが必要。
既定ではモデルファイルを不要とし、complete jobのassets/source/前後不変記録と凍結bundleを照合する。
`--models DIR`指定時だけ実model/charsetのSHAも再照合する。いずれもSessionを作らない。
分類対象の入力ID数とunique RGB数は別に数える。既存一回目の分類とcases全文が同じことを確認する。
