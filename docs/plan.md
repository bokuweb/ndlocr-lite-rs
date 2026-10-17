# ndlocr Python→Rust移植（ONNX）計画

## 目的
- `ndlocr/src/ocr.py` を中心に Rust 移植する。
- ONNX Runtime で CPU 推論を先に安定化する。
- 出力互換（xml/json/txt）を維持する。

## フェーズ
- Phase 1: CLI土台 + 推論I/O（前後処理、charset読込、smoke）
- Phase 2: `DEIM -> crop -> PARSEQ` 接続、カスケード、中間構造
- Phase 3: 出力互換（JSON/XML/TXT）
- Phase 4: 読み順整序
- Phase 5: 回帰・性能・配布

## テスト方針
- test/document first
- t_wada方式（Red->Green->Refactor）
- 効果的な箇所で property-based testing

## 2026-10-18: 外周近白紙

- 仕様 `docs/nearwhite-panel-paper.md`、局所Red 2件と後段色縁処理のRed 1件を先に固定。
- 全成分250以上の4近傍経路を保護し、既存色縁除去でも元RGBを維持する。囲まれた近白字形と249境界を検査。
- 固定28画像の全文比較と公開pin統合はomo #79で実施。全画像の精度改善・JPEG一般化・production readyは未確認。
