# 強い2行に覆われるDEIM複数行候補

仕様は `../../deim_duplicate_extent.md`。比較基準 c6d4674、モデルは
`../../upstream-assets.json` の固定公式モデル。文字列や正解は検出選択に入力しない。

- `real-red.log`→`real-green.log`: 実画像で対象2 crop→1 crop。
- `synthetic-red.log`→`synthetic-green.log`: 合成/cascade/PBT 3失敗→成功。
- `boundaries-green.log`: 信頼度・距離・近白画線・抑制の連鎖を含む6件成功。
- `detections-{before,after}.json`: 生検出51件は同一、選択33→32。
- `selection-audit.json`: 除去した大矩形だけを特定。他32検出のmetadataは同一。
- `default-tests.log` / `onnx-tests.log`: 通常479件・実モデル381件成功。
- `clippy.log`: 対象Clippy成功、既存17警告（新規変更箇所なし）。

`cli/` の4画像×既定DEIM/明示bands×2版、全16プロセスが成功。
既定DEIMの神戸市だけ生111→75、width64→28、strict95→59/1034文字。
他の7組は本文byte同一。全24画像別指標が非悪化。WERも同CSVに残す。
NDL資料は真値と範囲を推論前に固定し、初回対照後は開発用回帰とする。

raw/selected JSONだけから、勝った2つのcropに入らない全画素の復元を主張しない。
画素判定は量子化差のcollarを許す。JPEG/低解像度・未知資料・速度は未確認。

再現:

```sh
python3 tools/fetch_upstream_assets.py models --offline
cargo test --test multiline_duplicate_tests --test duplicate_line_detections_tests
NDLOCR_INTRA_THREADS=1 \
NDLOCR_TEST_DEIM_MODEL=models/deim-s-1024x1024.onnx \
NDLOCR_TEST_PARSEQ_MODEL=models/parseq-ndl-24x256-30-tiny-189epoch-tegaki3-r8data-202604.onnx \
NDLOCR_TEST_CHARSET=models/NDLmoji.yaml \
cargo test --features onnx --test deim_extent_regression_tests -- --include-ignored
```

CLIのargv・binary hash・stderr・終了状態は `cli/runs.json`。truthは同梱evalの4件。
`tools/eval_ocr_metrics.py --require-all --normalize none` / `width` / `strict` へ
`cli/{deim,bands}/{before,after}` を渡すと再採点できる。lock hashは `selection-audit.json`。
CIのworkflow scope制約は前フェーズと同じ。新規構造テストは通常CIが実行し、追加の
実モデル対象はローカルで実行した。既存CIそのものの設定は変更しない。
