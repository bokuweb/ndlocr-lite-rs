# 25/45字のCascade境界を調べる固定対照

23/24/43/44字のASCII大文字alphabet巡回＋ `」/』` の8文字列を、角ゴシック/明朝28px・
PNG/JPEG75・余白8pxでOCR前に固定した。32画像とliteral truth、計64 memberを `corpus.zip` に保存する。
認識結果を見て画像/真値/条件を変更しない。generator/font/Pillow/画像/truth/archive SHAと寸法はmanifestにある。
fontは配布せず、通常テストにローカルfont/Pillowは不要。再生成時だけ記録したmacOS fontが必要。
Rust fixtureテストは全組合せ・真値・member・decode・寸法を検査する。

実APIではこの画像群の新しい経路変更は0で、モデルが出した正読24組の括弧欠落も残った。
成分の接触等で今回の支持条件を満たさない。合成開発対照で未知実資料holdoutとは数えない。
モデル/再認識の精度を、純粋な25/45字の画線支持テストだけから主張しない。
全結果は `docs/benchmarks/body-closer-recovery/`、先行仕様は `docs/body_closer_recovery.md`。
