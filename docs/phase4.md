# Phase 4 仕様（test/document first）

## スコープ
- 読み順整序の純粋関数を追加
- `run_page` 出力 `lines/texts` に整序を適用
- 水平行/垂直行の最小ルールを定義

## 受け入れ基準
- 水平行: `Y` 昇順, 同値時 `X` 昇順
- 垂直行: `X` 降順, 同値時 `Y` 昇順
- 縦横混在: 垂直行を先、水平行を後（各グループ内は上記ルール）
- `run_page` で `texts` が整序後の順序に一致
- `cargo test` / `cargo check` 成功
