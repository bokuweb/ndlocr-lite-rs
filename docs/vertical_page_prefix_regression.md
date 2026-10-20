# 実Page経路のprefix回復を固定する回帰テスト

## 実装前仕様

Oの旧Page検証はbaselineと同じ実行物を使っていたため撤回した。新しい実行物とP1の
記録版はknown bold31で末尾まで一致したが、全444入力の採用gateはまだ完了していない。
この取り違えを実モデルのPage経路でも見つけるため、CIでskipしない固定PNG回帰を追加する。

- 新規holdoutではなく、既知Nのuncertain_bold_31_pad0原PNG/GTをそのまま固定する。
- 隔離したe5 baselineへ先にテストとPNGを追加し、Page本文の末尾欠落でRedを確認する。
  同じテストを変更しないO sourceへ置きGreenを確認する。製品条件は変更しない。
- PageRecognizerを公式DEIMとPARSeq30/charsetでloadし、eval/defaultを実行する。
  結合本文が固定GTと完全一致し、finite confidence、正の画像内bbox、元RGB不変を検査する。
- 不正RGB後に同じPageRecognizerを再利用し、同一設定で本文/bbox/平均bitsが再現することを確認する。
  cross-platformで固定confidence bitsやDEIM浮動小数点由来の固定bboxを要求しない。
- モデル/画像がなければ成功としてreturnせず失敗する。既存ONNX lib --include-ignored CIを使い、workflowを変えない。
- 実行物はfresh=falseと独立SHAを固定して保存する。共有Cargo targetのstale再利用を受け入れない。
- root sourceを固定して走っている444入力測定には変更を加えない。まず隔離コピーで実行し、
  全Page gateが成功してからrootへテスト/fixtureだけを接続する。gateが失敗したらこのproposalも隔離したまま扱う。

既存Fragment fixtureはrawと通常poolを保つ別の回帰であり、このPageテストの代わりにはしない。
