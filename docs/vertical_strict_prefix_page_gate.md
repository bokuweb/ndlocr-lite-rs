# OのPage/API/CLI回帰gate（実行前）

完全prefix延長Oのnative runtimeを固定し、既存293ページ、既知縦書き7ページ、N/N2新規144対照を残す。
既知7のID/RGBが既存293と重なる場合はIDを衝突させず、同一入力は同一として出典を記録する。
新規対照を精度結果から選ばない。GTはPageRecognizer・CLIへ渡さない。

各画像についてPARSeq30/50/100のPageRecognizer eval/defaultの6経路と、CLIのDEIM/bandsの2経路を比較する。
CLIが追加する最後のLFだけを1個除去して比較し、原GT/本文をstripしない。
line数/bbox/confidence bits/本文の差と、入力別none/width/strict文字・語の差を記録する。
入力別退行0を必要とし、改善0でもそのまま記録する。

既存293のbaselineにはH公開後の全8経路の実記録を再利用してよい。ただしe5とのsrc差が
cfg(test) module宣言だけで、製品runtime/依存/モデル/画像/設定が一致することを機械検査する。
既知7と新144のbaselineは今回の正確なe5製品から再測定する。
候補側は全画像を同一最終O runtimeで測定する。過去のN candidate出力は使わない。
プロセス完了receipt、source/model/input/binary SHAと新規/再利用呼出し数を残す。
Page API・CLIプロセス数を内部NN forwardとして数えない。未計測forwardはnullとする。
