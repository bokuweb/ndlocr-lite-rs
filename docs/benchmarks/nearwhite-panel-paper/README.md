# 近白紙の先行回帰

`spec-before-implementation.md` と `red-tests.rs.txt` を先に固定し、完全白のみの保護では例示/PBTの2件が失敗した（`red.log`）。最小失敗は `[250,250,250]`。外周探索を各成分250以上へ拡張後、後段色縁除去による `[255,254,254]` → `[255,255,255]` の上書きを新しい先行テストで再現した（`fringe-red.log`、`fringe-red-tests.rs.txt`）。色縁除去にも同じ訪問マスクを渡して保持する。

`panels-green.log` は新旧パネル/色縁12件、`final-green.log` は外周探索4件が成功する。後者のintegrationは名前filterにより0件であり、integrationの根拠には使わない。`final-clippy.log` はCI同等のClippy成功（既存6警告あり）、fmt成功。固定全文比較はomo側の新しいnearwhite-paper記録を参照する。局所成功だけから全文精度改善は主張しない。
