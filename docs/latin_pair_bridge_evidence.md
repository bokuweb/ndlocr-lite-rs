# Latin pair案の再現証跡仕様

採否に関わらず、元画像・真値・native描画記録・有効RGB・実入力capture・
全raw出力・全入力別の採点表・凍結sourceとcommand・失敗ログを保持する。
親は狭い丸括弧packet（162,915組）とし、追加60,228組を独立標本とは数えない。
新規ページ精度測定は行わない。

mediaは相対path/内容SHAでindexし、同じbytesのaliasも元pathとSHAを保つ。
各ZIPは50MiB未満に分ける。製品の採否が決まる前に静的mediaをpackしてよいが、
NN実入力captureや結果を処理途中のまま完成扱いしてはならない。
復元時は全archiveとmember/aliasのhashを検証し、モデル不要で画素・真値・
全入力別採点・raw/mean/API/CLIの証跡を再照合する。実Rustの保存raw再生と
tensor照合も、計測source/実行binaryのSHAを固定して別に検証する。

## 作業用領域の同一内容ファイル

独立復元の空き容量を確保する場合、完了済み評価の生成済みtxtだけを対象に、
内容SHAごとのcanonical fileから強制copy-on-write cloneで置き換えてよい。
通常のhardlinkにはせず、別inodeと書き込み分離を先にテストする。
元path/内容SHA/サイズ/permissionを保持し、置換前後のbytesを全件照合する。
画像・RGB・native描画・source・model・archive・実行ログは対象にしない。
clone未対応、symlink、同時変更、内容不一致では置換を停止する。
この処理は私的作業領域の容量調整で、再現scriptの必須依存にはしない。
