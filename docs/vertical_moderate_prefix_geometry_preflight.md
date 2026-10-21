# R対照の字形境界をNN前に検証する

Rのruntime/認識はFVC v2接続後まで待つ。新対照のNN0準備として、既存の凍結test-only
geometry実行物を再利用し、全72画像×3モデル寸法の216入力をsupported二分割へ通す。
NN/GT/glyph boxをcollectorへ渡さず、実RGBと入力寸法だけを与える。

字形bboxを持つbody/repeatedの5,256 glyphについて、両経路の全切出し境界が
実ink bboxの内部を横切っていないことを独立に調べる。全part RGBも元座標と比較する。
任意の字形cutを見つけたらR全体を追加NN前に保留し、対照やsplit閾値を合わせ直さない。
既存O/Hや製品の結果を、この診断だけで変更しない。

初回描画はENOSPCで停止したがNN0。69 PNG、68完全RGBと1部分RGBを保持した。
閉じた自分の実験データのみをSHA/復元台帳付きで可逆圧縮してから、出力先だけ変えて72を再描画した。
69 PNG/68 RGBは完全一致、部分RGBも新RGBの同じprefixだった。生成条件/GTは変更していない。
