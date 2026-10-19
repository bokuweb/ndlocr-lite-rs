# 語中closer案の公開接続検証

`docs/internal_detached_closer.md` の固定条件を変更せず、旧99,591組・新5,088画像×3モデル・
元Horizontal56画像×3モデルの全none/width/strict文字/単語指標が非悪化で、
新NNでも実際の適用がある場合だけ、公開APIのRedから製品接続へ進む。

- 既知 `情)報` 原画像で、純粋な単行製品処理と実Sessionが `情報` となるRedを残す。
  raw/候補は `情)報` のまま、fragmentも既存処理を保持する。
- 単行最後のfallbackだけへ接続する。前段の端closer・包含・1画素・分離部品の条件を変更しない。
  source/binary/model/inputを認識前に固定し、再構築後の実binaryで検査する。
- 旧新計115,023保存raw組を製品へ再生し、全件が凍結案と一致すること。これは新規NN数と区別する。
- 公開APIは全変更の画像/該当モデルに加え、noiseのpalt+gap+paper/8+JPEGフラグの和が偶数の384画像、
  5closer・両側空白1・紙8の全240本文画像、元Horizontal全56画像を全3モデルで確認する。
  重複ID/モデルは一度だけ数え、raw/本文/mean f32 bit、RGB不変を検査する。
- 同じfixtureでSession/Pool/並列batch/単一Session batch/standalone、raw/候補/fragment、
  エラー後の再使用、Cascade開始3/2/1を全3モデルで確認する。追加forwardはない。
- 実CLIは全変更に加え、全2本文・3font・2サイズ・4shapeについて
  `(palt,gap,paper,JPEG)` が `(0,0,0,0)/(1,1,8,1)/(0,1,8,0)/(1,0,0,1)` の192負例を全3モデルで実行する。
  末尾LFを厳密に1個だけ除き、本文/空白を固定値と照合する。
- 全通常/ONNX CI対象/example、fmt/diff/Clippyを実行する。ページはfragment用poolと
  関係関数のbyte一致を確認し、新規ページNNを行わない場合はその限界を明記する。
- 全件比較・公開API・CLIのいずれかで退行があれば接続を止め、反例を保存する。
  同じ結果へ支持条件を合わせ直さない。証拠はSHA付きで保存し独立復元も検証する。

## 最終検証の記録

固定条件を変更せず製品接続まで完了した。全115,023組に文字/単語指標の退行0、
生完全一致89組・width完全一致121組を回復した。出力変更136組のうち5組は全指標同点。
実API2,176組・実CLI712件、通常661件/ONNX593件/example23件、新規Clippy警告0。
新規ページNNは行わずfragment経路のsource不変性を検査した。
詳細・限界・失敗も含む証拠と再現は
[internal-detached-closer](benchmarks/internal-detached-closer/README.md) を参照。
凍結した先行仕様とsourceは同packetにも保存する。
