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

## 2026-10-18: 全角ASCIIに隠れる字形退行

- `docs/width-sensitive-evaluation.md` と5テストを先に固定し、4失敗を確認。
- `eval_ocr_metrics.py --normalize width` は全角ASCII94文字だけを写像し、空白・大小文字・他の互換文字を保持する。既存の指標と併記する補助採点で、認識本文には適用しない。
- 標準ライブラリのCLI/全範囲/既存互換テストを3OSのCIで実行する。モデル30/50/100の同一chunk比較はomo #79で行う。

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

## 2026-10-18: 原chunkの余白感度

- `docs/margin_sensitivity.md` と原RGB fixture欠落のRedを先行し、製品の全ページ出力を再現して6件を取得。非白RGB保持と前処理tensorの変化を検査する。
- 右側の白列追加probeもRed→最小実装→Green。通常fixture1件・probe3件、全60条件×2推論の再現性を検証し、Ubuntu/macOSの実モデルCIにも同じprobeを追加。
- 白2列で正読 `and culture.` が `Cand culture.` になる反例を固定。既知6件の信頼度選択0誤りは事後分析であり、精度改善・未知資料への汎化・本番採用を意味しない。次は選択条件を先に固定した別資料の全件非悪化評価。

## 2026-10-18: 原/縮小候補の信頼度比較（明示opt-in）

- `docs/confidence_margin_candidates.md` を先に固定し、7テスト中6件のRedを確認。有限信頼度比較PBT・不正RGB・推論失敗・同点・同一入力の1呼出しを7件Green化。
- `infer::margin_candidates::recognize_horizontal_margin_candidates` は原画像を先に認識し、幅/高さ5倍以上のchunkだけ縮小限定余白を比較する。本文は変更せず、平均信頼度が厳密に高い場合だけ候補を選ぶ。
- 通常pipelineは変更しない。未知資料6画像+既知8画像の製品接続検証・追加推論時間の測定はomo #79で実施する。精度改善・本番採用は未確認。

## 2026-10-18: 信頼度選択の追加回帰監査

- 14画像の生の合計827→819に対し、全角ASCIIだけを折り畳む補助評価では770→763。合計改善でも既知2chunkの `AI` → `Al` が残り、画像別/既知字形の非悪化条件は失敗した。
- `selected()` のRustdocと比較仕様へ実測した制約を追記。通常pipelineと選択処理は変更せず、自動選択の製品採用は保留する。
- 次は同じ原chunkを公式30/50/100文字モデルで比較し、分割条件を分離する。既知14画像は開発用回帰として扱い、未知資料での精度改善とは称さない。

## 2026-10-18: 候補解析の実入力capture

- `docs/candidate_input_capture.md` と未実装Red4件を先行し、任意RGB往復PBTを含む4件をGreen化。候補exportへ `--chunk-input-dir` を追加し、同じ認識直前RGBを可逆PNGへ保存する。通常pipelineは変更しない。
- omo #79の既知正読対照3帯/226字、30/50/100モデル計49区間で、過去の本文・区間・寸法を再現。同じ区間の保存RGBはモデル間でも同一。50の引用符置換は正解が第2候補、50の括弧欠落1件と100の引用符欠落2件は固定位置の置換だけでは復元できない。
- 生の誤り0/2/2に対し上位10候補oracleは0/1/2。これは原文を参照した上限で、実装済み補正器の改善ではない。画像には対象記号が残るが、モデル重みと内部resizeの影響は未分離。本番モデル・製品pinは維持する。
- exampleの4テスト・通常Clippy成功。厳格Clippyは未変更の本体9警告で失敗し、ログをomo側へ保存。次は独立資料の小記号/色/書式/解像度の対照を先に固定し、欠落を含む認識側改善を検証する。

## 2026-10-18: 面積平均縮小の比較診断

- `docs/area_resampling_probe.md` と細線の未実装Redを先行。最小Green後に分数境界・BGR・拡大互換・入力拒否・濃度保存PBTの4 Redを確認し、5件Green化。truth/重複ID/不正座標拒否のRedを追加し、example全6件Green。
- `compare_parseq_resampling` は横書き同一RGBに既存bilinearと縮小軸の面積平均を適用し、同一CPU sessionで実行順を交互にする。全chunkの本文・信頼度・寸法・区間・前処理/推論時間・失敗を保存する。新規公開APIや製品pipelineの変更はない。
- Ubuntu/macOSの実モデルCIへ6テストと既存candidate exportとのbaseline本文一致smokeを追加。omo #79で既知14画像、事前固定した合成36行、記号退行3帯×3モデルを比較する。合成対照は未知実文書holdoutではなく、実験だけで精度向上・本番採用を主張しない。

## 2026-10-18: 本流 uint8 OpenCV 前処理の比較診断

- `docs/opencv_preprocessing_probe.md` と通信境界の未実装 Red を先行。
- 診断専用 Python oracle を接続し、既存 bilinear と同一 RGB・同一 session で比較する。製品 Rust・公開 API・通常依存は変更しない。
- 不正入力・途中 EOF・応答長・非有限値を検査し、独立した6種類の画素 fixture で BGR/NCHW と丸めを照合。Ubuntu/macOS CI で実モデル経路も検査する。
- omo #79 の既知14画像・既存合成36行・記号3帯に対し、推論前に固定した全件非悪化条件で評価する。未知実文書 holdout と pure Rust 実装は別途必要。

## 2026-10-18: 近接した太字2行の検出

- `docs/narrow-line-gaps.md` と実資料/PBTの2失敗を先行。1pxの谷と少数文字の接触を、通常高・本文高・幅で限定して分離する最小変更。局所3件・既定lib136件・ONNX lib145件成功。通常Clippy成功（既存9警告、変更箇所なし）。
- 気象庁の見出し2行をnaive/fastとも分離し、縦画で結ばれた単一字形・探索上限の対照を保持。実資料は初回観測後の開発用回帰。
- omoの公開pin統合は保留。既知14画像と新規診断5画像で全文/正読対照を比較し、悪化がないことと見出し欠落の改善を要求する。未知資料への一般化・production readyは未確認。
