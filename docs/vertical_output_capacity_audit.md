# 縦書き残差の出力容量監査

## 追加実装・メタデータ読取・追加NN前の仕様

縦列容量回復f3f828dの前後比較を終えても、既知縦書き7入力のPageRecognizer30（eval）には
73/277 scalarの誤りが残る。ordinary 5、numerals 33、kana 13、手書き18、center_one 1、
center_two 1、center_three 2。既存の明示縦列APIの検査とは経路や紙準備が異なるため、
これを出力容量だけが原因だと断定しない。Hの固定条件を既知結果へ合わせて緩めない。

最初はモデルを変更せず、固定済み3モデルのinput/outputのdtype・次元とcharset長を記録する。
同じbackendへモデルをloadするが、Session.runや画像認識を呼ばない。モデルSHAを前後照合し、
動的/不明な次元もそのまま残す。モデル名の30/50/100から出力上限を推定しない。

既存decoderは[N,T,C]（または単件[T,C]）の各Tで最大1 Unicode scalarを追加し、EOSまたは
低confidenceで早期停止する。Tが正の固定値、Cがcharset長+1と一致する場合に限り、
raw出力scalar数の上限Tとして扱う。rank不一致、T/C不明、0、class不一致や変換overflowは
上限不明とし、仮の数字を埋めない。charset長+1とT*Cのusize計算もcheckedで扱う。batchの動的次元とTの動的次元を混同しない。

この静的上限の抽出は正/legacy2次元/動的batch、動的T/C・不正rank/class/0・overflowの
小さいRed→Greenを先行する。元worktreeの製品を変えず、別コピーへ診断だけを追加してよい。
これはメタデータ監査で、追加NNや精度改善には数えない。

次の段階で、上限到達・EOS・低confidence終了、検出crop、紙、内部白帯/分割境界を
既存保存データから区別する。新しい分割候補や閾値を試す場合は別仕様と未認識の対照を
先に固定し、全入力別文字・単語退行、余白/図形の安定性、実モデルRedを再確認する。
f3f828dの2採用・棄却例は開発回帰として保持する。新候補はこの文書時点で未実装である。

## メタデータ監査の結果（画像認識なし）

静的上限抽出はstubの2失敗/1成功から3成功へ進め、64例PBTで動的batchと動的Tの違いを検査した。
モデル/charset/source/binaryのSHAを固定して同じbackendへ3モデルをloadし、画像を渡さずメタデータを読んだ。
モデルとsourceは実行前後で不変、Session.run呼出しと画像認識は0回である。

| モデル | 入力（N,C,H,W） | 出力（N,T,C） | raw scalar上限 |
| --- | --- | --- | ---: |
| 30 | [1,3,24,256] | [1,31,7142] | 31 |
| 50 | [1,3,24,384] | [1,51,7142] | 51 |
| 100 | [1,3,24,768] | [1,101,7142] | 101 |

すべてFloat32・batch1。charsetは7141 scalarでC=charset長+1に一致した。
これは全frameを使った場合の上限で、EOSを出す余地や低confidenceの途中停止を改善した結果ではない。
既存7入力のPage30本文はordinary30、kana26、center各30、numerals7など上限より短い。
これらは既存後処理後の値なので、rawの停止理由を直接示すとは扱わない。

次の診断では既存7入力の全検出cropと、旧/追加42画像を結果で除外せず固定し、既存APIの
raw候補/step/停止理由を保存する。新候補ではなく、T到達・EOS・低confidenceと既存後処理の差を
区別する監査とする。追加NNを行う前にcrop/GTの対応と全出力の収録条件を先行固定する。

## raw停止理由監査の事前仕様（追加NN前）

対象はH監査の42個の原RGBすべてと、既知縦書き7ページの保存済みDEIM bboxから
無補間で切り出す全cropとする。3モデル×eval/defaultのbboxと順序が一致することを先に
検査し、想定の12cropと異なる場合はその不一致を保存して停止する。単列6ページのGTは
既存の全文、手書きの6列は既存GTの物理的な右→左の順と対応させる。CLI等の付加LFを
除去済みの保存GTは再trimしない。PNG/RGB/GT/bbox/元ページ/SHAを推論前に固定する。
合成フォント・手書き・旧NDLの既知開発回帰であり、新規holdoutとは数えない。

全54入力を30/50/100の順で既存のraw候補API、Auto、top_k=1へ一回ずつ渡す。
162 API呼出しを予定するが、内部Session.run回数はinstrumentationなしに推定しない。
各行のraw本文・f32平均bits・stop_reason・停止stepを含む全候補・UTF-8 byte rangeを保存する。
画像単位のエラーも保存し、失敗を隠して後続を選別しない。42入力は以前のraw本文/bitsと
完全一致を必須とし、追加12cropは既存Page本文と平均bitsとの差をすべて記録する。
モデル/charset/製品source/実行binary/manifestのSHAを固定し、実行後にも照合する。

出力上限と停止理由、低confidence停止step、後処理前後の差を集計する。
GTやこの診断結果を認識へ渡さず、候補閾値・分割・既存selectorを変更しない。
準備処理はGTの改行/空白保持、bbox不一致拒否、半開区間のRGB切出しをRed→Greenで検査する。
既存raw候補APIのEOS/低confidence/全frame消費・range・平均互換のテストも再実行する。

## raw監査の結果

準備3テストのRed（3失敗）→Green後、全6経路でbboxと列順が一致することを確認した。
既知7ページの12cropは、半開区間の行コピーとPIL cropがbyte単位で一致した。
元42RGBを加えた54入力を3モデルへ渡し、162 API呼出しすべて成功した。
内部Session.run回数は計測しておらず、API呼出し数から推定しない。

| モデル | EOS停止 | 低confidence停止 | 全frame消費 |
| --- | ---: | ---: | ---: |
| 30 | 27 | 27 | 0 |
| 50 | 38 | 16 | 0 |
| 100 | 36 | 18 | 0 |

既存42×3のraw本文/平均f32 bitsはすべて一致した。追加12×3の平均bitsも既存Page出力と一致した。
本文の差は30のnumerals末尾のspace1個のみで、既存後処理の差だった。trimして照合していない。
全162組のUTF-8範囲、選択token/確率bits、低confidence/EOS停止条件、f32逐次加算の平均を
独立に検査した。既存候補APIの5テスト（64例PBTを含む）も成功した。

30のordinaryとcenter3例は30文字を出した後、31frame目でEOSを選んだ。一方、余白/ジグザグ/
筆線などの負例にも最終frameでEOSまたは低confidence停止する例がある。従って「最終frameに
届いた」だけで分割候補を採用しない。numeralsと手書きcrop5は30で途中の低confidence停止、
kanaと残る手書きはEOSで終了しており、単一の出力上限説明へまとめない。
モデル/source/入力は実行前後で不変。この監査は精度改善や新規holdoutの評価ではない。

## 公開監査パケットの事前仕様

モデルやbinaryを配布せず、54入力/GT/元bbox、全162raw step、メタデータ、sourceと失敗ログを
圧縮保存する。復元スクリプトは入力SHAを検査し、モデル・NNなしでstep/平均の独立検査と
162行CSVを再生成してbyte一致を検査する。出力先は空ディレクトリに限定し、既存ファイルを
上書きしない。ZIP相対pathの逸脱と既存出力拒否を先行テストする。
