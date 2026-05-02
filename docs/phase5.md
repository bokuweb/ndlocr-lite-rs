# Phase 5 仕様（test/document first）

## スコープ
- 回帰フィクスチャによる出力安定性検証
- JSON/XML/TXT の生成結果を固定サンプルで比較
- 仕様変更時に差分が即検出できる運用土台

## 受け入れ基準
- 固定入力からの `build_ocr_json` / `build_ocr_xml` / `build_text` が fixture と一致
- 文字列エスケープを含むケースで比較する
- `onnx` feature 無効時の smoke 実行で案内付きエラーを返す
- `onnx` feature 有効時の smoke テスト雛形（`ignore`）を用意する
- `cargo test` / `cargo check` 成功
