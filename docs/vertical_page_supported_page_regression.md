# FVBの実Page回帰（実装前仕様）

全件比較を待つ間に、公開PageRecognizerの方向fallbackを固定する回帰を先に作る。
新規FVB先行対照 `fvb_vertical_arial_36_121_pad2` のPNGとliteral GTをそのままfixtureへ保存する。
既知font合成開発画像であり、独立実資料とはしない。GTは121 Unicode scalarでU+3000を含む。
先行FVBの返値は120 scalarで、この空白一字を欠落する。GTを変更せず、raw期待値も別に固定する。

公式DEIM/PARSeq30/charsetを必須とし、load失敗やasset不在を成功としてskipしない。
通常実行ではignoredとし、モデル付き実行で `--ignored --exact` を指定して実行1件を確認する。
公開APIのeval/default両設定について、以下を要求する。

- 一行のraw全文が先行FVBのliteral出力と一致し、非空白scalar全文がGTと一致する。
- 矩形が元画像内の非空領域、confidenceが有限の0..1である。
- 元のRGB bufferを変えず、不正RGB入力をエラーにした後も同じrecognizerで再利用できる。
- 再利用時の本文・矩形・confidence bitsを最初の成功結果と比較する。

隔離O sourceに同じテストを加え、水平分割出力による実失敗Redを保存する。
凍結FVB sourceの推論byteを変更せず、同じ回帰が実際に1件Greenになることを確認する。
旧ビルド再利用を避け、source mtimesの更新、freshな対象artifactと実行物SHAを記録する。
テスト成功だけで採用せず、別途全921 NN入力/922採点ID×8経路のgateと必要suite/CIを要求する。

ローカル容量が少ないため、このRed/Greenも独立Linux CIで両snapshotを直列にbuildして実行する。
新しいintegration testとfixtureだけを両sourceへ追加する。推論sourceのSHA不変と、freshな
non-test lib/CLI/Page artifact、freshなtest artifact、実行したtest binary SHAを記録する。
Redはexit101に加え、対象本文assertionによる「0 passed/1 failed/0 ignored」を必要とし、
asset不在・compile失敗・未実行をRed成立として扱わない。Greenは「1 passed/0 failed/0 ignored」
とexit0を必要とする。raw stdout/stderrとcommand/build記録をartifactへ残す。
