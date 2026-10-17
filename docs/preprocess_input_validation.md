# 前処理の入力検証とテンソル確保

PARSeq/DEIMの公開RGB前処理APIは、不正な寸法・バッファを`Result::Err`で返す。
テンソル確保前に、入力の両軸が正、`width * height * 3` がoverflowせず
RGB長と一致すること、出力の両軸が正であることを検証する。
出力の要素数とf32バイト数もchecked arithmeticで検査し、Vecの
`isize::MAX`バイト制限を超える場合は確保せず拒否する。
有効なサイズの確保要求はfallible reservationを使い、失敗をエラーとして返す。

PARSeqのAuto/Horizontal・バッファ書き込み型APIも同じ寸法契約とする。
検証で失敗した書き込み型APIは出力とscratchを変更せず、次の有効入力を処理できる。
DEIMの幅または高さ0を持つ元画像も拒否する。
有効画像のBGR/RGB順・補間・正規化・回転・padding・モデル入力は変更しない。

算術overflow、確保可能範囲を超えるサイズ、空軸、RGB長の不一致をRed化する。
不正入力後の再利用と有効画像の独立参照計算・PBTを維持する。
実際に巨大バッファを確保するテストやOSのOOM挙動に依存するテストは行わない。
これは総メモリ上限や、OSのovercommit後のOOM停止を保証するものではない。
