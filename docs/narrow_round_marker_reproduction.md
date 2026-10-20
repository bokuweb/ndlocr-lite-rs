# 狭いround profileの証拠の独立復元

公開packetは比較元1ec730aのowned-parts-body-bridge packetをSHAで固定して継承し、
追加画像・元RGB・有効RGB・実入力・真値・native trace・凍結source・全結果を保存する。
モデルやOS font fileは配布しない。portable復元は保存した画像とtraceの整合性を検査し、
NNやfontの再描画を行ったとは主張しない。元環境で行った648描画の再検査とは区別する。

復元では全archive/member/aliasのSHA、全162,915組の親raw/真値/比較元出力と元RGBを照合する。
追加8,064条件と同じ244元画像のHorizontalについて、原画/有効方向/atlas/実入力を照合する。
全2,304 pre-JPEG noise組は保存元nativeから独立に再構成して元画素保持・重なり0を検査する。
真値の構築と重複判定も再計算し、内部真値衝突を認めない。

native traceの検査はOS fontへのアクセスを要しない。
FreeTypeではBASIC/face0/palt未指定、実文字・font記録・size、全prefix advance記録の長さ、
文字ごとのglyph maskとmissing glyphでないことを照合する。
CoreTextでは引数とtraceのpalt/size/文字、NFCで統一したfont path、全runの同一font、
非0 glyphと全string indexの被覆を検査する。異なるpaltやfallbackのtraceは拒否する。
これらは保存traceの検査であり、portable環境のfont fileのSHAを測定したことにはしない。

旧11群198表ファイル、追加全群144表ファイル、Horizontal108表ファイルの計450表を
同梱した正式scorerで再計算し、CSV/Markdownのbyte一致と3つのaudit一致を要求する。
文字・単語の退行も全表の各行で検査する。モデル/条件/方向/同一RGBを独立標本として数えない。
実Session・CLIの全記録、raw/最終本文/f32 mean bit/画像の一致と末尾LF1個の処理を検査する。
復元した元RGBに対して、凍結した実Rust製品の全162,915組と8,064×3形状のtensorも別途再検証する。
