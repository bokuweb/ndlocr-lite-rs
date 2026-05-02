# Phase 3 仕様（test/document first）

## スコープ
- 出力互換（JSON/XML/TXT）
- XML 組み立て（`OCRDATASET` / `PAGE` / `LINE`）
- XML/JSON/TXT ファイル書き出しの統合
- `mock-page` から 3 形式を一括保存

## 受け入れ基準
- 出力モジュールの単体テストが通る
- `mock-page` E2E で `.json` / `.xml` / `.txt` が生成される
- XML は宣言行 + `OCRDATASET xmlns=""` を含む
- `cargo test` / `cargo check` 成功
