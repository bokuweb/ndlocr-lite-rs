# 広い白余白の分離: TDDと回帰

先行仕様は `docs/wide-blank-gaps.md`。`red.log` は3失敗・2成功、
`green.log` は5成功。PBTは48ケース。実失敗6帯は元画像のcropとhashを
`tests/fixtures/wide_blank_gaps/source.json` に保存した。

`all-tests.log`: 通常413件成功（doctest2件を含む）。
`onnx-tests.log`: CI対象と新規分割テスト328件成功、固定モデル・1スレッド。
`clippy-ci.log`: CIと同じ `cargo clippy --all-targets --no-deps` 成功。
`clippy-onnx.log` のONNX全targetも成功。fmt成功。
新規コードの警告はない。`clippy.log` の厳格な `-D warnings` は既存libraryの
6警告で失敗した。既存テストにもRangeInclusiveの警告が2件あり、抑制していない。

通常成功経路と `split_line` の全幅保存は変更しない。
実モデルでの全文改善・既存成功との比較はomo側の
`docs/benchmarks/ocr-2026-10-18-wide-spaces/` で公開commitを固定して行う。
この画素テストだけを精度改善の証拠として扱わない。

upstreamのmainstream監査は `upstream-refs.txt`。
前回からmaster/tagは変わらず、新規に取り込む差分はない。
