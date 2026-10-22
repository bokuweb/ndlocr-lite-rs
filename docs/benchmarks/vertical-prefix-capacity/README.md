# 縦列prefix延長候補: NN前の幾何検査で棄却

先行仕様: [vertical_prefix_capacity_proposal.md](../../vertical_prefix_capacity_proposal.md)。
既存Hの条件は変更せず、異なる二分割の全文一致と元raw prefix完全保持を要求する候補を
test-onlyで検証した。製品Page/Pool/Sessionへは接続していない。

新しい既知3書体の36本文と24図形を先行固定し、既知54RGBも残した。
既存index/fixture575ファイルに含まれる312,538 SHA値と新60画像のPNG/RGBは未重複。
同じ既知fontの合成開発対照であり、独立holdoutや学習未使用資料とは数えない。

全114×3形状の342幾何組を調べ、5原RGBで12個のmodel/route/glyph切断が見つかった。
隣接chunkの両端を数えたイベントは24。先行固定した全件gateに従い候補全体を棄却した。
新baseline/chunk認識・model load・Session.runは0回。結果で画像を除外した精度比較は行っていない。
151組の幾何planはdummy T-1文字で文字数gateだけ開いた数で、実raw候補の到達数ではない。

## モデル不要の再現

Pillowを利用できるPythonで実行する。Codexのbundled runtimeにも含まれる。

```sh
python3 -m unittest discover -s docs/benchmarks/vertical-prefix-capacity -p 'test_reproduce.py'
python3 docs/benchmarks/vertical-prefix-capacity/reproduce.py --output tmp/vertical-prefix-capacity-replay
```

出力先は存在しないか空のディレクトリとする。全ZIP/member SHAと全114画像/RGBを検査し、
独立PIL CCW回転とcropで1,151 chunkの全byte一致を再現する。各glyph bboxから24イベントを
独立に再計算し、Rustの失敗記録ともbyte一致する。342行CSVと集計JSONは保存版とbyte一致を要求する。
モデルload/追加NNは0回。`verification.json`は空の復元先での実行receiptである。
途中の復元でディスク容量不足になった履歴も保存する。SHA一致した生成済み復元コピーだけを
整理し、異なる/途中のファイルとログはgzipへbyte保持した。元画像/raw/GT/source/binaryは削除していない。

`audit.zip`は全画像/GT/glyph bbox、幾何chunk、先行仕様/source/Red/Green/失敗と独立監査を収録する。
モデル・font・binary本体を含めずSHAを保存する。復元用readerは前段停止監査のreaderを継承し、
既存出力保護/path逸脱の2テストも実行した。prototype6テスト（64例PBT）とdefault library400件が成功、
35件ignored、新規Clippy警告0。幾何検査の失敗は意図的に保存し、テスト成功や改善と数えない。
次は分割境界の一般的な修正を別の先行仕様と未認識対照で検査する。
