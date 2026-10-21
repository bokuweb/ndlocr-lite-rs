# S0: LowConfidence停止部分を細かい二分割で観測する（追加計測前仕様）

R0は既知H後欠落3組を、既存supported/Otsuの主・副分割から22 unique crop/2model sessionで
raw Horizontal/top-k1認識した。これは新しい回復政策を本体へ接続する実験ではない。
主副合意から新規補完を支持する結果は無かった。

`q_serif_50_pad14`/model30では、主分割の一部分がLowConfidenceで10 scalarで止まり、
平均confidenceは約0.983だった。主連結43 scalarは現在Fragmentと同じ欠落を含み、
副分割は全てEOS停止でGTと同じ50 scalarになった。平均confidenceだけで完全な認識とみなせない。
他2組は主副とも同じ欠落のまま、又は読点/中点が不一致だった。

## 既知1組の観測のみ

- R0で唯一LowConfidenceが観測された上記1組だけを開発診断対象とする。
  GTによる全体誤り分類から始めた既知例であり、新規holdoutや採用根拠とはしない。
- 元PNG/RGB/全glyph bbox/現在Fragment結果を変更しない。supported/Otsuのspan divisorを
  **2と4**へ事前固定し、二つの異なる分割を記録する。主divisor1の採用条件を後から緩めない。
- 元と同じRust split helperを使うモデル不要記録器をtest-only sourceへ追加し、
  各chunkのRGB/矩形/元glyph内部切断を保存する。切断一件でもあれば追加NNへ進めない。
  実RGBを独立PILの無補間回転/cropと照合する。モデルload/NN呼数は0。
- 同一model/RGB/寸法/方向/top-k1/sourceの既存R0副分割結果だけを再利用できる。
  新しいunique cropはSHAで固定してから、既存frozen raw Horizontal記録器で全件認識する。
  GTや欲しい文字をNNへ渡さず、失敗・停止・実入力RGBを残す。
- 両連結全文のbyte合意、全部分EOS停止、実T未満のscalar数、finite0.9..1 confidence、
  現在Fragmentの全文字/空白を保った一意な内部連続挿入の可能性を診断する。
  いずれかを満たさなければ、この組に対する安全な補完支持なしと記録する。

この診断で支持が得られても本体へは接続しない。新政策を作るなら、H採用かつ実LowConfidence停止を
観測した場合だけ調べ、元H結果を全て保持する別仕様・先行Red/Green・未測定対照・全既知gateを必要とする。
S0/Q/R0の失敗結果や、Q新72の改善0を成功データへ置き換えない。
