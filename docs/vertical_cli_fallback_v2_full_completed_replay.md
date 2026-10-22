# FVC v2全件完了証跡の独立再計算

run 39593424953 / head fe0e359371fb7ca7b289092103f6d7f16d41d62f の
56ワーカーと比較jobが全件成功してから、APIの59個の元artifact ZIPを保存する。
サイズ/SHAをAPIのmetadataと照合する。build stageはnativeも含めて全bytesを照合し、
公開packetには4実行物を除いたstageとbuildログ、56個の元worker ZIP、比較ZIPを含める。
省略したnativeのbytes/SHAを明記し、公開読取器がnative blobを再認証したとは主張しない。

既存のcomplete-readerへFullBundleV2を渡し、1,311物理入力/1,317採点入力を使う。
26,220画像経路、79,020 CSV行、39,510正規化込み比較を独立距離で照合し、
本文/stdoutのframing、実argv、Page bboxとf32 confidence bits、全採点aliasを再検証する。
欠損/途中完了/別buildのjobは拒否する。全体の退行0、Page変更0と明示的full_gate_passを
要求する。これらが揃うまではRoot runtime patchを適用しない。

再計算には認識モデル・NN・フォントを必要としない。
Root適用後のfresh実行物・CLIと品質処理の回帰・必須CIは別の採用条件として継続する。
