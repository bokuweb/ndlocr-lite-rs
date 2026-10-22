# P5 / Q / R0 / S0 / S development observations

本体推論はO/1f240beのままです。**Q/Sとも新72 nativeで改善0のため保留**しています。
既知例の成功を新規対照の成功と混ぜず、FVBの別の全件比較にも合算しません。

- P5: 羅生門3列×6設定の18 Page結果が従来Oと本文/矩形/mean bits一致。
  実PARSeq forward30試行/成功、18 primary/12 part RGBを独立照合しました。
- Q: 一意な内部挿入だけを許す隔離案。既知羅生門1列のPageはRed1→Green1、
  元ASCII空白を残して文字距離6→1/語2→2。新72の全3,456画像結果は前後不変でした。
- R0/S0: 既知残留誤りを選んだ診断です。R0の22 raw cropでは安全な内部延長を支持せず、
  S0の既知1組では細かい両routeが全文で合意しました。
- S: H採用と実LowConfidenceを必要とする細分化案。既知例で7文字回復をRed1→Green1で確認し、
  ONNX非ignored462成功/115 ignored。新72の全3,456画像結果は前後不変でした。

Q/Sはそれぞれ新72（本文54/図形18）×3model×8native経路を前後とも新たに認識しました。
各案10,368公式CSV行（none/width/strict）の文字/語誤り・分母を独立検算し、悪化0・改善0でした。
Q/Sとも大きい全既知native/Page/CLI採用比較へ進めていません。

`index.json`は11,828論理ファイルを2,837内容アドレスへ結び付け、`development-v1.zip`の
全byteとmemberをSHA256で検査します。ソース・Red/Green・失敗・新入力・観測値・採点を含みます。
元の実行物SHA/fresh artifact/固定入力の記録は保存しますが、モデル・font・native実行物は含めません。
一部の実行物はprivateでlossless gzipへ保存しているため、古いprivate監査を再実行する場合は
各`closed-experiment-binary-archives.json`に従い元のpath/byteへ復元する必要があります。

大きいQ/S geometry ZIPはprivateに残し、公開packetには全50,653 memberのSHA/サイズを含めました。
元PNG/RGBと保存境界からPILで独立回転/切出しを行い、全cropのSHAが捕捉byteと一致するか再検算できます。
Qは26,506 crop・supported切断0/legacy156、Sは24,147 crop・supported切断0です。
ZIPの圧縮byteを再生成することと、crop RGB byteを再生成することは区別します。

リポジトリrootからモデル不要で検算できます（Pillow12.3.0で確認）。

```sh
python3 -m pip install -r tools/low_stop_evidence_requirements.txt
python3 -m unittest discover -s tests -p test_vertical_low_stop_evidence.py -v
python3 tools/verify_low_stop_evidence.py
```

readerは全archive/member、前後nativeの入力網羅・本文/mean・batch一致・対象外経路不変、
公式CSV全20,736行、PNG/RGB、glyph境界、捕捉RGB SHAを検算します。新しいNNは実行しません。
packet内の旧private driverには当時の絶対pathや実行物を要求するものがあるため、
公開検算には上記readerを使ってください。新対照は既知fontの合成開発資料であり、
未知資料全体の改善率を示すものではありません。固定案/採否の詳細は以下です。

- [Q結果](../../vertical_unique_internal_results.md)
- [S先行仕様](../../vertical_low_stop_recovery.md)
- [S結果](../../vertical_low_stop_results.md)
- [保存仕様](../../vertical_low_stop_development_packet.md)
