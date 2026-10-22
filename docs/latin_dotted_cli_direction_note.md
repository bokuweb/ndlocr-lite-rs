# 点付きLatinのCLI方向対照の補足

## API/CLI検証実行前

固定raw候補と実製品の282,465組一致を確認した。
公開APIの426組は変更102組（Auto76、Horizontal26）と事前固定noise324組である。

既存の単行CLI `recognize` にHorizontal指定はない。全件を同じ方向でCLIへ渡せるという
先行仕様の前提を修正する。Horizontal26組は公開Pool APIで検査し、CLIでは同じモデル・
元RGBのAuto26組を追加する。Autoの期待本文は既に保存・採点した全件結果を使い、
Horizontal期待値を代用しない。したがってCLIも426プロセスだが、APIと方向構成が異なる。
製品へ新しいCLI方向オプションを追加せず、方向別の検証範囲を記録する。
CLI画像は凍結RGBのlossless PNGとし、JPEGを直接Rust decoderへ渡す同一性は主張しない。
