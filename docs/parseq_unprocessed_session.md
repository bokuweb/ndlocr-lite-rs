# 後処理なし PARSeq セッションの再利用

Phase 5 の複数ページ推論では、モデルと文字セットをロードした `ParseqSession` を
行・ページ間で再利用する。`recognize_rgb_u8_unprocessed` は既存の単発
`recognize_rgb_u8_with_score_unprocessed` と同じデコード本文・信頼度を返す。
反復文字削除や空白整理は行わない。従来の後処理付き API は維持する。

固定モデルによる回帰では、ロード元を削除しても反復数字を複数回認識でき、
不正画像の失敗後にも有効画像を認識できることを確認する。
モデルと画像は `NDLOCR_TEST_PARSEQ_MODEL` / `NDLOCR_TEST_CHARSET` /
`NDLOCR_TEST_REPEAT_IMAGE` で明示し、省略時はテストを成功扱いにしない。
