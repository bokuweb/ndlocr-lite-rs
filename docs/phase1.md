# Phase 1 仕様（test/document first）

## スコープ
- Rust CLI 雛形
- 入力画像列挙
- DEIM/PARSEQ の前処理・後処理（純粋関数）
- charset 読み込み
- 画像ロード接続済み smoke 入口

## 受け入れ基準
- CLI/IO/前処理/後処理/charset のテストが通る
- `cargo test` / `cargo check` 成功
- `onnx` feature有効時にモデルロードsmokeが動く構成
