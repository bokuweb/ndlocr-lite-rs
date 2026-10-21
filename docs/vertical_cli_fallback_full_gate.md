# FVC新72 gate後の全件比較（準備仕様、全件NN前）

新72の全4経路・3正規化で入力別文字/語悪化0と新本文raw文字改善/語非悪化が確認できた場合だけ進む。
現時点でその比較は未完了であり、この文書は採用を許可する結果ではない。
既知例のRed/Greenや幾何照合だけで本体へ接続しない。

全既知対照は、FVB凍結921物理/922採点、Q新72、S新72、FVC新72、追加実資料6物理/11採点を併せる。
合計1,143物理ID/1,149採点ID/1,142 unique RGB。元の重複RGBと全GT aliasを保持し、独立標本数とは主張しない。
原packetと各manifestのSHAを照合し、pathを移す場合は元pathと新pathの対応を保存する。
入力/GT/モデル選択・閾値・支持条件は観測後に変更しない。

隔離FVB基準と固定FVC候補を同じLinux/releaseで実測する。
CLI DEIM/bands×cascade on/offの4経路は9,144 CLI process/画像結果、
Page30/50/100×eval/defaultの6経路は13,716 API結果、計22,860画像結果となる。
全10経路・両variant・1,149採点ID・3正規化で68,940公式CSV行を独立距離と照合する。
CLIは入力別文字/語の悪化0を必要とし、Pageは本文・行数・矩形・confidence bitsの完全一致も要求する。
新72の改善数は全件比較で別の新規改善へ二重計上しない。

分割実行では全ID×経路×variantのCartesian集合を検算し、重複/欠落/部分jobを拒否する。
source/lock/model/charset/入力/GTと異なるfresh実行物のSHA、実argv、stdout framing、failed/partialを残す。
一度のfresh buildを分割jobへ配る場合も、元build receipt/source/binary SHAを全jobに引き継ぎ、
全jobで実行物/入力/assetを前後照合する。Page load/sessionとCLI process/内部forwardを混同せず、未計測はnull。

padding/min confidence/binarize threshold/quality boost、feature無効、入力エラー後の再利用も、
設定の保持を純粋testと少数の固定実モデル対照で検証する。全件gate後に必要suite/実モデルCIを通し、
最後に本体へ最小差分で接続する。新72で悪化または新本文改善0なら、この高コスト全件比較を行わず全FVCを保留する。

## 新72完了後・全件NN前の範囲更新

新72は576結果/1,728 CSV行を完了し、全288入力/経路組の3正規化で文字/語悪化0、
14本文画像のraw30経路組が改善した（bands on/off各14、DEIM on/off各1）。この先行gateは成功。
その間にC1新96の入力/GTがNN前固定されたため、「全既知」へその96も追加する。
C1の認識方針はFVC sourceへ混ぜず、単なる既知入力として採点する。

全件は1,239物理ID/1,245採点ID/1,238 unique RGBとなる。
CLI4経路の前後9,912 process、Page6経路の前後14,868 API結果、合計24,780画像結果、
公式none/width/strictの74,700 CSV行を独立検算する。以前の1,143/68,940の数は旧scopeとして保持する。
新72の改善を再び新規改善へ数えず、追加全件で一つでも文字/語悪化があれば採用しない。

入力と全採点aliasをorigin付きで結合し、同じRGBの既存別IDも保持する。重複ID/未採点/未知alias参照を拒否する。
7 kind（Page30/50/100、CLI DEIM/bands×on/off）×8完全shardを計画し、完了集合をCartesianで検証する。
Page kindは両variant×eval/default、CLI kindは両variantを実測する。未完了jobを成功へ数えない。

fresh releaseの前後buildを一つのLinux jobで行い、source/lock/build stdout/own artifact/CLIとPage SHA（CLIは前後で異なること）、
必要なONNX runtime共有libraryを凍結する。分割workerは同じ凍結buildをSHA検査して使い、path移送をreceiptへ記録する。
各workerは実asset/入力/GT/実行物を前後照合する。old FVB原bundleとFVC adapter原packetをSHAで参照し、
baselineの281 bound filesが元FVB候補と一致し、追加差が実CLI testだけであることを確認してからbuildする。

Pageは不変を検証する経路なので、両own artifactがfresh=falseならPage実行物の同一SHAも許す。
CLI変更案にPage用check_binary_pairを流用せず、Pageは前後とも実測して本文/矩形/confidence bitsを全件比較する。
