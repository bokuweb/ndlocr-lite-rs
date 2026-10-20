# 縦列容量回復の証拠packet仕様

製品での全回帰判定が完了してから公開packetの完了状態を作る。入力固定・元画素と支持maskの分離・
H1不成立・H2有理数Otsu・候補と棄却・実モデルRed/Green・既存/追加の全結果・容量不足による
ビルド失敗と再試行を保存する。モデルと実行binary本体は配布せず、sourceとSHA・呼出し記録を保つ。

入力archiveは先に作れるが、NN結果のarchiveは全293ページ行列の完了receiptと全入力/経路の
coverage、文字/単語非退行、固定画像/真値/source/model不変を確認するまで完了としない。
元JPEG、全PNG/RGB、真値、生成全点集合とseed/source、既知7縦書き、293画像、42行画像を含める。
重複byteはSHAでaliasにできるが、全元パスを復元し、alias先の欠落や同じパスの異なる内容を拒否する。
1 ZIPを50MiB未満にし、各memberとarchive全体のSHAを記録する。別の空ディレクトリへ復元して検査する。

モデルなしのPython再現では、正式CSV/Markdownの再生成・独立した距離計算・元JPEGからのcrop、
図形の全点集合/回転、支持maskの独立有理数閾値・全byte、全chunkの元RGB/座標、全保存API/CLIと
confidence/矩形の対応を確認する。欠落した表や入力/経路を見逃さないcoverageテストを先行する。

さらに同じ製品sourceの別コピーへpure replayテストだけを加え、全42入力×3モデルの126判定を
実際のRust selectorで再実行する。callbackは保存raw chunkとf32bitを返すだけでNNを呼ばず、要求された
cropの全RGB/寸法と保存chunkを照合し、全callback数・採用/棄却・本文/mean bitを確認する。
真値をselectorへ渡さず、unsupportedや棄却も含める。元worktreeとNN比較用source凍結は保持する。
これは製品選択の再実行であり、追加NNや新しい精度標本とは数えない。
