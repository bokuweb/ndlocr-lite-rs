# 保存済みAuto/Horizontal比較

先行仕様は [auto_orientation_saved_audit.md](../../auto_orientation_saved_audit.md)。
以前の128画像と今回の64画像について、全3モデルの保存rawを同じ原RGB・真値・IDで対にした。
新規NN forwardは0。既知の横組みで、余白によってAuto回転した部分集合だけを扱う。

生文字距離は1,902→196、単語距離576→188、生完全一致388組回復。
none/widthの文字・単語指標に悪化0だが、strictでは100モデルの2組の文字距離が2→3へ悪化した。
同じ原画像のpalt=0/1で、真値`)情理`に対してAutoは`1,0`、Horizontalは`理問題`だった。
どちらも正読ではなく、全正規化非悪化とは表現しない。方向の一律変更は製品へ採用していない。
真の縦組み・枠・ノイズを含む新対照と一貫した前処理/後処理方向の検証が別途必要。

`evidence.zip` は先行仕様・pairingのRed/Green・driver・全36採点表・全固定結果と反例を含む。
`parents.json` は原画像/NN結果の既存4archiveをSHAで参照する。モデル/font/binary本体は含まない。

```sh
python3 docs/benchmarks/auto-orientation-saved/reproduce.py --output tmp/orientation-restored
```

参照archiveと使用member/aliasのSHAを確認し、原RGBと真値を照合してから、
正式scorerの36表ファイルと全18群の集計を再現する。モデルとPillowは不要。
