# 実primary列の二分割安定性を診断する

実装/追加NN前仕様。追加stop probeで認証した18 model/RGB（元36 Page条件）をすべて対象とする。
既存のOtsu支持画像とsupported splitter（span divisor 1/2）を、raw長/EOS/H採用に依存せず
診断専用で呼び出し、元RGBのlossless切出しを保存する。製品のH/O/Q/Sの条件は変更しない。

collectorはtest-onlyとし、任意manifestのモデル寸法・元RGBだけを受ける。GT/raw期待本文を渡さない。
分割不能も全件に残し、成功列だけへscopeを縮めない。認識用画素はOtsu支持画像へ置換しない。
独立座標対応と元入力不変を先にRed→Greenで確認し、全モデルの幾何をNN0で凍結する。
その後、異なるモデル/寸法/RGBの全partを既存候補collectorへ渡し、同じpartのみdedupする。
実part RGBと元座標を独立照合し、全36 originに対応づける。

二分割の文字列完全一致、空白、信頼度、元rawの完全prefix/部分列関係をそのまま報告する。
一致するような空白除去/文字置換は行わない。既知の正解は採点と診断にだけ用いる。
この段階は改善policyの採用ではなく、新規精度改善数は0。
