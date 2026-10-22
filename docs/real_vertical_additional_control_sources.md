# 追加実資料の選定状況（OCR未実行）

[先行仕様](real_vertical_additional_controls.md)に沿って、既存二冊以外の実スキャンを調べている。

- 樋口一葉『一葉全集 : 校訂』32版、博文館1911年。NDL PID1088069。
  NDLはインターネット公開・pdmを記録している。
  [書誌・公開範囲](https://ndlsearch.ndl.go.jp/books/R100000002-I000000554589)、
  [IIIF manifest](https://dl.ndl.go.jp/api/iiif/1088069/manifest.json)。
  TOCの本文リンクはcanvas5だが、実画像は目次だった。最初の本文はcanvas6の左ページで、
  題字/柱を除いた右から3列を認識前に選んだ。3物理画像・6採点ID（第3列の屹/吃と嘘/噓の別表記4通りを含む）を固定した。
- 芥川竜之介『地獄変』、野田書房1936年。NDL PID1908126。
  NDLはインターネット公開・pdmを記録している。
  [書誌・公開範囲](https://ndlsearch.ndl.go.jp/books/R100000002-I000002206891)、
  [IIIF manifest](https://dl.ndl.go.jp/api/iiif/1908126/manifest.json)。
  最初の本文はcanvas7の左ページ。右から3列を選び、3物理画像・5採点IDを固定した。
  第1列の樣/様、第3列の德/徳を認識前の別表記として残した。
  羅生門とは別書籍・別組版だが、同じ著者である。
- 梶井基次郎『檸檬』、東京楽譜出版社1946年。NDL PID1133993。
  NDLはインターネット公開・pdmを記録している。
  [書誌・公開範囲](https://ndlsearch.ndl.go.jp/books/R100000002-I000000691736)、
  [IIIF manifest](https://dl.ndl.go.jp/api/iiif/1133993/manifest.json)。
  canvas5は扉、canvas6が本文。列の傾きによって全文字を含む矩形へ隣列が入るため、
  単列の切り出しを未確定として残した。この資料のGTは固定していない。
- 島崎藤村『春』緑蔭叢書2、上田屋1908年。NDL PID887860。
  [書誌・公開範囲](https://ndlsearch.ndl.go.jp/books/R100000002-I000000512449)、
  [IIIF manifest](https://dl.ndl.go.jp/api/iiif/887860/manifest.json)。
  インターネット公開・pdmを確認したが、本文の下部に退色・欠損があり、最初の3列を
  確信を持って転記できなかった。原画像を保存し、GT未固定・採点対象外とした。
- 候補の『宮沢賢治全集』第3巻/PID1236401は公開manifestの取得が404だったため採用しなかった。
  [NDL書誌](https://ndlsearch.ndl.go.jp/books/R100000002-I000000776121)の個人送信案内とも区別し、認証経由の取得は試していない。

取得原JPEG・manifest・書誌HTML、HTTP response/日時/SHA、選択矩形とPNG/RGB・GTは
`tmp/real-vertical-additional-controls/`へ保存している。一葉のGT固定は
`ichiyo-first3-GT-freeze-v2.json`、地獄変は`jigoku-first3-GT-freeze.json`。
一葉v2は採点器の制約に合わせてID・ファイル名をASCII化したもので、原v1のGT・画像bytesは同じ。
合計6物理画像・11採点IDを`additional-controls-manifest-v1.json`へ固定した。
PNG/RGBと原JPEGからの行単位の切り出しを照合し、既知1065物理画像との重複はなかった。
本文previewと各列の二度の目視は同じ担当による確認である。
独立専門家の注釈や学習未重複を意味しない。

追加資料のOCR NN/model loadは0。FVBの進行中全件比較へ混ぜていない。
追加資料を測る際は、原run/分割runと本体基準を整理した後、別の凍結runとして行う。
