# 追加実資料の実decoder停止理由を観測する

実装/追加NN前仕様。6物理列×30/50/100×eval/defaultの全36保存primaryを対象とし、
欠落が見えた列だけを選ばない。P5では停止理由が記録されていなかったため、既存raw候補collectorで観測する。
認識結果を変更するpolicyは追加しない。Q/S/H/Oの条件も変更しない。

入力は元Page traceの実primary RGBだけを使う。Auto回転対象の縦長primaryはlossless反時計回り90度へ変換し、
同じ既存preprocessのHorizontal経路へ渡す。全出力のraw本文/f32 mean bitsを元Auto結果へ照合し、
同一性が成立しない場合はその不一致を保持し、元Pageの実停止理由として断定しない。
モデル・寸法・RGBが同じ入力だけをdedupし、全36 originへの対応を保持する。
dedupのraw期待値が競合する、寸法/RGB長が不正、originが重複する場合はNN前に拒否する。
GT/元final本文/画像IDからの条件変更は行わず、NN manifestはモデルとRGB・寸法・SHAだけとする。

既存fresh baseline実行物とそのsource/build receiptを再認証して再利用する。
全入力/元trace/source/実行物/モデル/charset/driverをNN前に固定し、前後同一性を確認する。
collectorはtop_k=1でEOS/LowConfidence/TimestepsExhaustedと停止step候補を保存する。
dedupとlossless回転を先にRed→Greenで検証し、出力RGBも独立座標対応で確認する。
実行後に全36対応と生の停止理由を報告する。これは診断であり、新規精度改善数は0。
