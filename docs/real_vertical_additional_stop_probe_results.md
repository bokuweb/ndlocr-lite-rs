# 追加実資料のdecoder停止理由

全6物理列×3モデル×eval/defaultの36保存primaryを、モデルと実RGBが同じ18入力へdedupして観測した。
18 raw APIはすべて成功し、全36 originのraw本文/f32 mean bitsが元Page traceと一致した。
モデルsession loadは3、内部forward数は未計測。再検証時の追加推論は0である。

| model30入力 | raw scalar数 | 停止step | 実停止理由 |
| --- | ---: | ---: | --- |
| 一葉col1 | 23 | 23 | EndOfSequence |
| 一葉col2 | 22 | 22 | EndOfSequence |
| 一葉col3 | 11 | 11 | LowConfidence |
| 地獄変col1/2/3 | 各30 | 各30 | EndOfSequence |

model50/100の全12入力もEndOfSequenceだった。合計18入力はEOS17・LowConfidence1。
停止stepまでのtop1候補と実RGBを保存し、本文byte範囲、f32逐次加算によるmean、EOS token/
低信頼度条件、回転の全画素対応をモデル不要readerで独立検証した。

一葉col1/2の途中欠落を低信頼停止だけで説明することはできない。一方col3は実際に低信頼停止だが、
既存Sの前提であるH採用はなく、Sをそのまま接続できる証拠ではない。
EOSを無視する変更や、これらのGTに合わせた文字挿入・空白削除・閾値調整は行っていない。
次に検証すべき対象は、元画像で支持された別分割でどの範囲を安定して読めるかであり、
実rawと二分割の合意を保持できるかを診断する。新規精度改善数は0、本体はFVBのまま。

公開packet: `docs/benchmarks/real-vertical-additional-stop-probe/`。
before 4,052,792 bytes（SHA256 `4ac87777874dfbc994c0cd441d316557e41a0d47f80be776f11e97aa912c4cc2`）、
results 3,652,645 bytes（`3fe8c0225c26af7c2ed48df92038c1cae28bc039fcb5a1987f46670a8d27dbd7`）。
source bridgeのroot sourceコピーは実行後保存だが、NN前bindingと完全一致する。
補助処理3 Red→3 Green、packet reader未実装Red→4 Green。以前のtraceで不明だった停止理由を、
実出力一致のある今回の観測で補った。元traceの欠けていた情報やcontainer byteの再構築を主張しない。
