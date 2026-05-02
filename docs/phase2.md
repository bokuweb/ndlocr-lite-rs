# Phase 2 仕様（test/document first）

## スコープ
- カスケード振り分け（3/2/その他、25/45閾値）
- line bbox crop
- `detections -> crop -> cascade` 接続
- `run_page` 骨格（件数 + line中間構造）
- JSON構造組み立て
- JSONファイル書き出し

## 受け入れ基準
- 各モジュールの単体テストが通る
- `cargo test` / `cargo check` 成功
