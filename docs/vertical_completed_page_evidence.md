# 完了済みLinux Page結果のモデル不要再検証

FVB原runの30/50と分割retryの100を、CLIの未完了分から分けて保存する。
Pageだけの成功を全件gate成功とは扱わない。原packet/data/source/閾値は変更しない。

公開packetはouter ZIP・全memberの圧縮JSON index・SHAを持つ。安全な相対pathだけを展開し、
Page30/50の各1 jobとPage100の4 shardを既存readerで検証する。
全source/build/argv・認識前plan・入力coverage・frozen後のstampを必要とする。
models/weights/native/fontを実行・取得せず、凍結Page結果だけを使う。

公式CSVはモデルごとのnone/width/strictの9表・計33,192行を原GT/保存本文から独立再計算する。
入力別文字/語の悪化0とraw改善組30:35/50:35/100:8を確認する。重複モデル/設定を独立画像に数えない。
Page保存結果11,052件、PageRecognizer12load/model session24loadは過去の実測範囲であり、新しいNNではない。

モデル選択診断の424組・99入力ID/99RGBもmodelなしで再現し、保存した全casesと一致させる。
raw-limit超過はH/O採用のproxyで、実CLI内部traceではない。14prefix組は3入力に重複する。
中断CLI runの成功した1 command/24scalar出力は診断領域だけへ置き、完了artifactやgateへ混ぜない。

実packetを読むintegration testを先行し、verifier未実装の失敗を確認してから作る。
公開verifierは任意の新出力dirを受け、過去結果を上書きしない。full_gate_passは常にfalse。
CLIの4分割と全44,256CSV行・実Page回帰・本体bridge/必要QAは引き続き別に必要とする。
