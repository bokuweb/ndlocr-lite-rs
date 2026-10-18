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

## 2026-10-18: 左右の補間色外縁（検証中）

- `docs/lateral-panel-fringe.md` を先に固定し、左右階段縁の例示/PBTと孤立色文字による追跡中断の Red を確認。
- 混合色・外周紙・長い連続境界で制限した最小実装で局所8テストがGreen。罫線・厚み・長さ・囲まれた字形・紙RGBを負例で検査。
- 実画像+2度は現時点で14行のまま。固定28画像の認識・速度比較と公開pin統合は未実施。production ready・16行回復は未達。

## 2026-10-18: 切り出し境界の連結成分（検証中）

- `docs/clipped-band-components.md` と失敗する境界回復・不正入力テストを先に追加し、明示呼出しの汎用APIでGreen化。
- 四辺・斜め連結・独立図形・接続罫線・複数帯・新規矩形衝突を例示と独立探索oracleのPBTで検査（9テスト）。
- 既定検出経路は変更しない。固定80画像の画素保存と28画像の全文認識をomo #79で比較中。離れて欠落した文字・同形図形の意味分類・production readyは保証しない。
