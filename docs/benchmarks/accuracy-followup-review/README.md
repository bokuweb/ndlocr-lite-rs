# 最終残差レビュー（追加NN0）

- `known6-pixel-diagnosis-v2.json`: 公開controls `7c4aa9bd…` の6 RGBから再計算。
  `python tools/diagnose_vertical_scan_support.py` で同じJSONを出力する。
- `known6-saved-output-review-v1.json`: 元公開packetのSHAを列挙し、50/100 rawとCLI DEIMの
  byte一致、FVC v2全件内の実資料44 raw比較を保存。11採点IDは6物理列の表記aliasを含む。
- `red-v1.stderr` / `green-v1.stderr`: 診断helper未実装4 Red→4 Green。
  `python -m unittest discover -s tests -p test_additional_real_scan_support.py -v`。

初回private探索表は1px拡張のclampを省略しており、公開v2の採用表へは使っていない。
この診断は方向判定の一要素で、Rust採否traceや新規認識・新規改善を表さない。
