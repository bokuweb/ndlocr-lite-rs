# FVBの本体統合（全件gate後、実装前仕様）

凍結run39532673209は完了した。原runのPage30/50とretryのPage100/CLIを合わせた
10完了artifact・4種類・921 NN入力/922採点ID/920RGBについて、公式44,256CSV行と
22,128比較行を手元でも再計算し、remote結果と完全一致した。入力別文字/語悪化0、
新規本文raw7入力/経路組改善、全3,684CLI本文不変だった。
`all_four_kinds_complete=true`が全種類のcoverageを表す。実job数10なので古い
`all_four_jobs_complete=false`というfieldは4jobでないことを示すだけである。

- 現在の278 bound filesはO/1f240beと一致する。既存実PageのO Red/FVB Greenと、
  FVB純粋13 testの先行Red/Greenを採用前の根拠とする。
- 凍結candidateの推論5ファイルを移す。src/infer/vertical_fallback.rsのtest用capture module
  だけを除き、13 geometry/output-support testsを保持する。formatterの変更は許すが、
  本番codeの判定・数値・crop・callbackのtokenを変えない。
- CLI/CachedPageRecognizer/native H/O、dict/rule-pack、文書固有補正は変更しない。
  FVCのpadding/adapterやcascade observerはこの統合へ含めない。
- 本体CIの実モデルsuiteに既存vertical_fallback_page_testsを加え、固定PARSeq30/DEIMで
  literal本文・eval/default・不正入力後再利用を必須実行する。モデル未取得のskipはない。
- local fmt/default libと、default/ONNX Clippyの既存警告比較、CIの全通常/実モデルsuiteを確認する。
  容量制約がある本体実モデルの全suiteはLinux/macOSのCIで検証する。
- Root Cargo.lockはignoredのまま。source bridgeと全件のrawCLI/CSV/失敗・成功を公開保存する。
  元の中断runや診断用partialをfullgateへ混ぜない。

統合後の追加実資料6画像/11採点表記は、このFVB本体を基準として別の凍結runで測る。
FVCおよびOの既知CLI cascade traceも独立した計画・gateで進める。改善作業全体は継続する。
