# 縦列のraw停止理由監査

先行仕様: [vertical_output_capacity_audit.md](../../vertical_output_capacity_audit.md)。
比較元はf3f828d。製品認識処理は変更せず、cfg(test)診断だけを追加した。

3モデルの静的出力は[1,31,7142] / [1,51,7142] / [1,101,7142]、charsetは7141 scalar。
これらはraw scalarの上限であり、文字を必ず上限まで出す保証ではない。
メタデータ読取はモデルload3回、画像認識/Session.runは0回。

元42RGBと既知7ページの全12 DEIM cropを各モデルへ渡した162 API呼出しは全件成功。
内部Session.run回数は計測していない。EOS停止は30/50/100順で27/38/36、
低confidence停止は27/16/18、TimestepsExhaustedはすべて0件だった。
最終frameで停止する負例もあり、上限到達のみを新しい回復条件にしない。
新規holdoutや精度改善の結果ではなく、既知開発データの診断である。

## モデル不要の再現

```sh
python3 -m unittest discover -s docs/benchmarks/vertical-output-capacity -p 'test_reproduce.py'
python3 docs/benchmarks/vertical-output-capacity/reproduce.py --output tmp/vertical-output-capacity-replay
```

出力先は存在しないか空のディレクトリを指定する。標準Pythonのみで、ZIPと全memberのSHA、
162組のUTF-8範囲・停止条件・確率bits・f32逐次加算平均を独立検査する。
既存42×3組の本文/平均bits、Page12×3組の平均bitsも完全一致した。
rawとPageの本文差は30 numeralsの末尾space1個だけ。trimして照合しない。
再生成する162行CSVとanalysis.jsonは保存版とbyte一致を要求する。モデルload/追加NNは0回。

`audit.zip`は54入力とGT、元bbox/ページ、全162raw steps、メタデータ、先行仕様、source、
Red/Green・Clippyと実行ログを含む。ONNX/binary本体は含めずSHAを残す。
各入力の旧/追加・合成/実資料・余白/図形の区分はmanifestのcohortを参照する。
同一書籍/既知fontの開発対照であり、モデル学習との重複は不明。

静的上限3テスト（64例PBT）、候補decoder5テスト（64例PBT）、準備3テスト、復元境界2テストが成功。
Clippyは既存45箇所と同じで新規警告0。通常full suiteはメタデータ段階の394 library件成功の
ログを保存しており、raw収録後に全スイートを再実行したという意味ではない。
空の復元先での成功receiptは`verification.json`に保存する。
