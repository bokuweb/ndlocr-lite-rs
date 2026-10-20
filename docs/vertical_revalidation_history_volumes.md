# Page再検証履歴の可逆分割アーカイブ

## 実装前仕様

P1の661ケースだけで元ZIPは約94MBに達した。全864ケースのZIP、ログ、sourceを
単一ZIPに詰めるとGitHubの単一file上限を超える恐れがあるため、履歴payloadを分割する。
原画像・ZIP・source・失敗を削除したり再符号化して小さく見せない。

- 既存content-addressed backing_objectsを再利用し、新規payloadは元byteのSHA256名で保存する。
- 各volumeの非圧縮payload合計が90,000,000bytes以内になるよう、追加前に次volumeへ移る。
  生成後には各実ZIPが95,000,000bytes未満であることも確認する。
- 同じSHAの再追加は同じ参照を返し、volumeを増やさない。既存archiveの参照も維持する。
- empty payload、容量ぴったり、次の1byte、重複、単独超過をRed/Greenで先に固定する。
  小さな長さ列の全組合せで、payload保存と分割予算を検査する。
- 元history-indexと全旧payloadを残す。readerは参照のarchive名を使って全byte/SHAを照合する。
- 重み・font・native実行物は追加せず、その既存SHA/bindingを保持する。

この変更は実験履歴の保存方法だけを扱い、OCRの政策・本文・測定数を変えない。

最初の80MB設定では、元audit-details.json（88,915,707bytes）の単独payloadで拒否された。
この失敗と部分ZIPを保持したうえで、公開時の明示budgetを90MBへ変更する。
95MB未満という実ZIPの上限は維持し、元JSONの再符号化や削除で回避しない。
これは保存容量の設定変更であり、OCRの採用条件を変更しない。
