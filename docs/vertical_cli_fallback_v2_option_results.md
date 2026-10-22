# FVC v2 設定384条件の完了結果とモデル不要再検証

固定済みv2計画の8画像×6設定×DEIM on/off×cascade on/off×前後2系統を完了した。
384 CLI process、1,152 CSV行、576比較で文字/語の悪化は0、reject_allの64 processは空本文だった。
これは既存設定を保持する回帰gateであり、新規精度改善の証拠へ重複計上しない。
内部forward/model session数は未計測である。最初の構文エラー実行は画像認識0として元計画とともに保持する。

実装前仕様: `verify_options` は公開ZIPから全command receiptを固定計画と照合し、本文/stdoutのLF、
48系統×8画像の採点用コピーとGTを認証する。none/width/strictの全4整数指標を独立再計算し、
576比較全体と悪化リストを再構成する。before/new72/plan/parser証拠とのSHA連鎖、
実行前binding、完了後一致記録を検証し、不完全実行を成功として受け入れない。
未実装readerのRedから始め、再検証自体ではモデルを実行しない。

公開結果は`docs/benchmarks/vertical-cli-fallback-v2/options-cli-matrix-v2.zip`（942,303 bytes、
SHA256 `5636fcfefa20ff2c4203c0f22ac67679cb9dcd6774d016d607779c8a00932da5`）。
reader未実装による1 ImportError Red→6 packet tests Greenを保存した。
`python tools/verify_vertical_cli_fallback_v2.py --options`でモデル不要再検証を実行できる。
各実行物のfresh build・source・入力は事前packetへ遡って認証するが、内部forward数の推計はしない。
