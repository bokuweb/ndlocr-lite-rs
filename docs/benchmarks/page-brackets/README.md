# 行断片の閉じ括弧を保持する

先行仕様 `../../page_bracket_context.md`、基準 c75d80d、固定公式モデルは
`../../upstream-assets.json`。画素・crop・モデルを変えず、ページOCRにだけ断片方針を使う。

- `fragment-red.log`→`fragment-green.log`: 未実装のAPI Red→単体/PBT 3成功（従来後処理10件も成功）。
- `cli-red.log`→`cli-green.log`: 生モデルに存在する閉じ括弧を既定CLIでも保持するRed→Green。
- `runtime-green.log`: PageRecognizerの両構築経路で本文とconfidenceが一致。
- `pool-compatibility.log`: 従来Poolのscalar/両batch/エラー回復、ページ並列を含む5件成功。
- `default-tests.log` / `onnx-tests.log`: 通常482件・実モデルを含むONNX対象387件成功。
- `clippy.log`: 変更箇所の新規警告なし、既存17警告で通常Clippy成功。

5資料×DEIM/bands×2版の20プロセス成功。`cli/` に全本文・全指標・実行argv・binary hashを保存。

| 資料 | DEIM 生 | DEIM width | bands 生 | bands width |
| --- | --- | --- | --- | --- |
| 神戸市お知らせ（1034字） | 75→74 | 28→26 | 80→78 | 32→29 |
| NDLアクセシビリティ（1058字） | 60→60 | 55→53 | 53→53 | 48→46 |
| 気候変動資料（800字） | 61→60 | 50→49 | 63→62 | 52→51 |
| 既存スキャン（226字） | 9→9 | 7→7 | 9→9 | 7→7 |
| ロゴ（8字） | 1→1 | 1→1 | 1→1 | 1→1 |

実際の値は `cli/*-none.csv` / `*-width.csv` / `*-strict.csv` が正本。
全30画像別文字指標が非悪化。6組の本文変更は閉じ括弧の挿入だけ、4組はbyte同一。
`cli/character-edits.json` はSequenceMatcherで全編集を独立に検査したもの。
追加の気候変動資料は4資料での方針確定後、真値・範囲を推論前に固定して初回比較し、
以降は開発回帰として扱う。同じ発行元の別資料であり、学習重複や未知資料全般は保証しない。

`onnx-concurrent-build-failure.log` は、通常とONNXを同じtargetで並行ビルドしてCLIが
onnxなし実行ファイルへ置き換わった失敗。default完了後にONNX binaryを再リンクし、
全対象を順次再実行して成功を確認した。A/Bは別名の固定binaryを使用しており影響なし。

再現:

```sh
python3 tools/fetch_upstream_assets.py models --offline
cargo test --test fragment_sanitize_tests --test postprocess_tests
NDLOCR_INTRA_THREADS=1 \
NDLOCR_TEST_PARSEQ_MODEL=models/parseq-ndl-24x256-30-tiny-189epoch-tegaki3-r8data-202604.onnx \
NDLOCR_TEST_CHARSET=models/NDLmoji.yaml \
NDLOCR_TEST_DEIM_MODEL=models/deim-s-1024x1024.onnx \
cargo test --features onnx --test page_bracket_context_tests -- --include-ignored
python3 tools/eval_ocr_metrics.py --truth-dir docs/benchmarks/page-brackets/cli/truth \
  --system before=docs/benchmarks/page-brackets/cli/deim/before \
  --system after=docs/benchmarks/page-brackets/cli/deim/after --normalize width --require-all
```

truthを `cli/truth/` に凍結し、元ファイルとhashは `cli/truth-manifest.json`。
`bands` と `none`/`strict` も同様に再採点できる。通常/ONNXの検証ビルドは順番に行う。
CIのworkflow scope制約は前フェーズと同じで、設定変更は含めていない。
