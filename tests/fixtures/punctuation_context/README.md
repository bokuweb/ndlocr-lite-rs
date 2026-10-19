# 句読点の余白候補の回帰画像

`docs/punctuation_space_component_context.md` の追加認識前仕様で生成した576対照からの回帰pin。
追加の学習/評価用holdoutとは数えない。PNGはJPEG75保存後のRGBを可逆で保持する。
生成仕様・font SHA・全候補は句読点空白ベンチマークを参照。manifestに出典ID/画像SHA/真値と
対象モデルの元本文・3候補・Rust f32 confidenceを保持する。

- joined30: 既知ページの丸ゴシック24px/半分から取得した原chunk、`測定場所、時刻、`。
  Fの画像支持がある実際のページ対象区間をpinする。前段Dの旧pinも全評価対照と実行時fixture archiveへ保持する。
- real_space30: 丸ゴシック24px、紙追加なし、`受付番地、 通知先` の実空白を保持する負例。

- joined50: 明朝20px、JPEG75後のRGB、`登録内容、保存先`。
- joined100: 角ゴシック20px、PNG、同じ本文。

末尾2例は50/100文字モデルの回復をpinし、3モデルへの混在bucket振分けも検査する。

- condensed_real_space100: 角ゴシック24pxを横65%にし、PNG保存した実空白の反例。
  `案内方法、 連絡先` の正読がDで誤読になるため、通常APIで空白保持を検査する。
