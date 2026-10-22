# 縦列の余白と容量guardの保存結果診断（追加NN・新方針前）

Oは実rawが選択T−1/T文字に達したときだけ二分割合意を検査するが、geometryには元画像の
height*input_height > 2*width*input_widthを必要とする。このwidthは余白/Rubyを含む。
追加実資料の幅・列高から余白依存を疑ったが、当該実資料をまだ認識していないため
現在の誤認識原因と断定しない。raw前処理やguardの変更もまだ提案・実装しない。

最初に公開Q/S packetの保存baseline72+72だけを使い、3モデルの固定metadataと実raw本文長から、
容量付近かつ元寸法guardで拒否される組を数える。GT誤りと現在Fragment結果は診断用に比較し、
GTを採否条件へ使わない。実際のH採用/実T一致などを直接観測したtraceとは区別する。

- packet/member/source/model/入力の結び付きを既存readerで検査する。
- 元のraw、Fragment結果、GT、入力のPNG/RGBとモデルmetadataを変更しない。
- scalar長、末尾Unicode空白、元寸法の整数不等式を記録する。
- 候補があればNNなしで実RGBと空白/Rubyを含む支持範囲を調べる。背景/文字を補完しない。
  Otsuや支持幅を再実装する際はRustの閾値・比較・tieを先に照合する。
- これは候補分類であり、新しい回復方針、追加NN、新しい採用結果ではない。
  変更を支持する根拠がなければ保留する。後続の案は別仕様・Red/Green・新対照・全件gateを必要とする。
