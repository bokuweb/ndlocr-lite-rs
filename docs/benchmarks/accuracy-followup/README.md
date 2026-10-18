# PR #21 継続: 下線付き長行

基準 main b490ade。仕様・出典・凍結真値は `../../accuracy-followup.md` と
`../../../tests/fixtures/eval/jma_kobe_notice.json`。固定モデル/charsetは
`../../upstream-assets.json` のSHA256で検証した。通常Rust依存lockは未追跡。

- `initial-red.log`: 分割エラーで停止する初回Red（画像下端の修正前）。
- `baseline-red.log` / `before.txt`: 下端修正後、分割エラー4帯だけ全幅1回認識する診断基準。
- `underline-red.log` / `underline-green.log`: 構造テスト3失敗→5成功。
- `underline-model-red.log`: 下線分割後も全文一致には未到達。
- `underline-model-green.log`: URL文脈回復の固定回帰が成功。
- `underline-after.txt`: 下線分割後の全文。行別比較は `underline-line-audit.json`。
- `underline-{none,width,strict}.{csv,md}`: 公開scorerの全指標。CER改善、生/width WER悪化を両方記録。
- `default-tests.log` / `onnx-tests.log`: 通常467件・CIと同じ実モデル対象362件が成功。
- `clippy.log`: Rust 1.99 Clippy成功。未変更箇所の17警告があり、strict成功とはしない。

URL4行は生 4→1 / 63→4 / 5→2 / 68→5文字編集。先頭28行は全文同一。
全文の改行・最終改行も含む生210→82/1034字、width161→33/1034字。
帯32件は不変、診断fallback4→0、推論数121→134。速度は未計測。
同系統の気象庁資料を追加した開発回帰であり、未知様式への一般化ではない。
通常の `recognize-page`/DEIM経路はこの分割APIを使用していないため、その精度改善も主張しない。

再現:

```sh
python3 tools/fetch_upstream_assets.py models
cargo test --test underlined_line_tests
NDLOCR_INTRA_THREADS=1 \
NDLOCR_TEST_PARSEQ_MODEL=models/parseq-ndl-24x256-30-tiny-189epoch-tegaki3-r8data-202604.onnx \
NDLOCR_TEST_CHARSET=models/NDLmoji.yaml \
NDLOCR_FOLLOWUP_OUTPUT=tmp/accuracy-followup/after.txt \
cargo test --features onnx --test accuracy_followup_tests -- --include-ignored
```

`NDLOCR_FOLLOWUP_REQUIRE_EXACT=1` を追加すると未解決の全文一致Redも再現できる。
真値と before/after を同名ファイルとして別ディレクトリへ置き、
`tools/eval_ocr_metrics.py --normalize none` / `width` / `strict` で再採点できる。

## 続き: 囲み側線と認識下線の棄却実験

`frame-red.log`→`frame-green.log` は一律除外案。`frame-model.log` で正読中点の退行が見つかり棄却。
`frame-close-red.log`→`frame-final-green.log` は近い枠を保持する限定案。最終本文は
`final-after.txt`、行比較は `frame-line-audit.json`、公開scorerは `final-{none,width,strict}.{csv,md}`。
基準から生210→76/1034字、width161→27/1034字。枠変更後も32帯、133推論。
中点を含む31行は前段の下線改善後と同一で、枠由来の1行だけ厳密改善。
最終通常473件、実モデル373件成功（`final-default-tests.log`, `final-onnx-tests.log`）。
Clippyは17既存警告、変更箇所の新規警告なし。

下線除去の実験は `underline-recognition-{red,candidate}.log` と
`underline-removal-probe.json`。区間とcrop寸法を固定し、URLのwidth誤りは
0/4/1/5→1/0/1/1。全件非悪化に失敗したため、通常処理に下線除去は追加しない。
`tests/underline_recognition_probe_tests.rs` はこの対照を再現する診断専用のignored test。
`NDLOCR_UNDERLINE_PROBE_REMOVE_RULE=1` で候補を、
`NDLOCR_UNDERLINE_PROBE_REQUIRE_EXACT=1` で未解決の完全一致Redを実行する。
その後の対照再現は `final-probe-{raw,candidate}.log` で成功。

## CI設定の公開制約

GitHubのOAuth tokenにworkflow scopeがなく、`.github/workflows/ci.yml` の変更を含むpushは拒否された。
CI設定変更を除いて公開する。`final-onnx-tests.log` は公開前のローカル検証で、既存CIの対象に
新規 `underlined_line_tests` / `framed_lines_tests` / `accuracy_followup_tests` を加えた373件。
既存CIの通常 `cargo test` は新規構造テストを実行するが、新規実モデル回帰はローカル実行のみ。
