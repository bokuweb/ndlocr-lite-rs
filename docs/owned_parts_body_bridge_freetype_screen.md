# FreeType BASICによる適用対照screen

## 実装・描画前仕様

raster監査で、同じmincho file/face W3/advance60でも、元fixtureはFreeType BASIC、追加対照は
CoreTextでink maskが異なることを確認した。元のPillow式で `情報」` の寸法/RGB bytesを完全再現できた。
支持条件は凍結したまま、元と同じ描画器で幾何到達性を調べる。

- pairの順序を `[情事,事情]`, `[情本,本情]`, `[情木,木情]`, `[情車,車情]`, `[情生,生情]` に固定する。
  mincho file・face index0・20px・本文末尾 `」`・PNGの10入力。
  全件を保持し、どちらかの語順で増分適用がある最初のpairを列挙順で選ぶ。
  元のCoreText対照10,944+16入力も保持し、NNの精度標本や独立holdoutへ数えない。
- Pillowは12.3.0/FreeType2.14.3、RAQMなしを記録し、明示 `Layout.BASIC` を使う。
  palt指定はサポートされないので、palt軸を作らず、常に未指定とする。
- ImageFont.truetype(path,size,index=0)、getbbox、余白8、ImageDraw.textの元式を使う。
  字形の伸縮・advance変更・橋追加はしない。元fixtureで画素一致するテストを先行する。
- 実font face名/layout/path/size/SHA、text/prefix/各文字の実測getlength、getbboxを記録する。
  numeric glyph IDはPillowから公開されないため、各文字の実getmask(mode=L)寸法/bytes SHAを記録し、
  未定義文字U+10FFFFのmaskと一致しないことを確認する。これをCoreTextの実glyph runと偽装しない。
- 凍結fc5/固定test-only案へ真値を仮rawとして渡す。幾何到達性だけで、NN/精度/改善数へ合算しない。
  全10画像で0なら条件を緩めず停止する。選んだpairの追加NN全組合せは別の生成前仕様へ固定する。
