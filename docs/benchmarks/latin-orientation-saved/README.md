# Latin追加対照の保存方向比較

先行仕様は [latin_auto_orientation_saved_audit.md](../../latin_auto_orientation_saved_audit.md)。
latin-pair-bridgeの19,440条件から、Autoが回転した横組み636元画像を全件選び、
保存済みの同じ元RGBのAuto/Horizontal rawを全3モデルで比較した。追加NNは0。
製品処理・方向判定・モデルは変更していない。

| 正規化 | 文字距離 Auto→Horizontal | 文字退行組 | 単語距離 Auto→Horizontal | 単語退行組 | 完全一致回復 |
| --- | ---: | ---: | ---: | ---: | ---: |
| none | 6,608→4,446 | 16 | 2,136→2,073 | 6 | 42 |
| width | 6,608→4,237 | 16 | 2,136→1,990 | 6 | 91 |
| strict | 5,957→3,572 | 2 | 1,908→1,785 | 0 | 123 |

全体の完全一致喪失は0だが、100モデルに入力別の退行がある。一律Horizontal案は採用しない。
text 252画像だけでもnone/widthで文字1組・単語6組、strictで文字2組が悪化した。
例として、真値 `） qjp` のAuto `0.00` に対しHorizontal `pration` は生文字距離5→7。
両方とも誤読であり、Autoが正しい方向と解釈しない。
真値 `）WM` に対する `0.00`→`( M` では生単語距離1→2となった。

残り384画像は前段のnoise対照で、body-only真値は診断上の規約である。
curve/forkは記号にも見え、意味が一意な非文字と扱わない。全27群集計と46退行指標行を保存した。
636条件×3モデルは1,908組で、重複RGB・モデル・描画条件を独立標本と数えない。
既知font・既知の横組みだけを扱い、真縦組みや未知資料に対する方向識別の評価ではない。
前の192画像監査とも別母集団のため改善数を合算しない。

pairingの4既存テストと画素回転の4新テストをRed→Greenで確認した。
元画像・真値・RGBのSHAと、Auto有効RGBが正確な反時計回り回転であること、
Horizontal有効RGBが元とbyte同一であることを全件検査した。
本文の空白と空文字は保持する。

evidence.zipは先行仕様、driver、テスト、失敗/成功ログ、全本文/真値、正式18 CSV/MD、
全集計と反例を含む。parents.jsonは親packetの必要6 archiveをSHAで参照する。
別の空ディレクトリで親から実画素・保存rawを読み直し、全18表と全集計/反例の一致を確認した。
実行記録とSHAは verification.json / execution/ に保存する。モデルとPillowは不要。

```sh
python3 docs/benchmarks/latin-orientation-saved/reproduce.py --output tmp/latin-orientation-restored
```

次の方向候補には真縦組み・短文・紙/枠/ノイズを含む新対照と、前処理/後処理方向の
一貫した契約が必要である。接触/部品/複数closerなどの後処理損失も引き続き改善する。
