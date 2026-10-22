# 検出された長い縦列の容量回復

基準8d9ae3d（runtimeはb9b64efと同一）との比較。PageRecognizerのDEIMが検出した極端に長い列で、
短すぎるraw結果と十分な画線支持がある場合だけ、同じモデルで元RGBの部分画像を追加認識する。
支持用Otsu maskはNNへ渡さず、従来のAutoと同じ無補間回転RGBを保持する。
一般的な画線/容量の条件だけを使い、書籍名・原文・字体辞書を選択へ渡さない。

| PageRecognizer30（eval/default各） | none/width文字距離 前→後 |
| --- | ---: |
| 旧3本文抜粋・9列・392 scalar（列間LF込み） | 269 → 236 |
| 追加3本文抜粋・9列・372 scalar（列間LF込み） | 209 → 177 |

採用は旧コマ5第2列と、条件を固定した後に初めて認識した追加コマ6第1列の2つの元RGBだけだった。
2設定の結果や同じ書籍の別列を独立文書として数えない。50/100とCLIの出力は全件同一。
本文GTからルビを除くという規約を明示し、ルビ・隣列ルビ片・紙の汚れを画素から消していない。

- 既存287画像＋実資料6抜粋の293入力を前後8経路で新規実行し、入力別none/width/strictの
  7,032文字・単語比較に退行0。本文変更は上記2抜粋×2設定の4組だけ。全行数・矩形は同一で、
  confidence bitの変更も4組だけだった。別の既知縦書き7入力も前後8経路・168比較がすべて同一。
- 旧/追加の全42行画像（本文18・余白6・図形18）を3モデル・5実API経路で照合し、630組の
  本文/f32 bitが固定selectorと既存後処理の合成結果に一致。余白・図形の216 Fragment経路組も従来値を保持した。
- 全42支持mask/閾値は独立有理数計算と全byte一致。NNなしの実Rust selector replayは
  全126判定・81保存callbackのRGB/寸法/本文/mean bitを再現した。
- 先行のH1支持案は独立groupが6〜22で40に達せず、追加NN前に棄却した。H2の数値条件は
  追加NN前から固定し、空白不一致を後から整形して採用していない。失敗・棄却も保存している。
- 通常762件、実モデルを含む718件、example24件が成功。Clippyは既存45件から新規0、fmt/差分チェック成功。

実資料はNDL『坊つちやん』（岩波書店1939年18刷、PID1884801）の6コマ。
取得時manifestのAccess RestrictionsはPDM、attributionは国立国会図書館。
[資料](https://dl.ndl.go.jp/pid/1884801)、[IIIF manifest](https://dl.ndl.go.jp/api/iiif/1884801/manifest.json)。
同じ書籍・既知書体の開発対照であり、未知書式のholdoutやモデル学習との非重複を主張しない。
図形の空GTは診断規約で、意味が一意な非文字正解とは断定しない。ここでは前後出力の安定性を要求した。

各ZIPは50MiB未満。元JPEG/PNG/RGB/GT/生成全点、H1/H2、raw/候補/棄却、source/モデルSHA、
TDD・PBT・NN呼出し記録・全採点表を収録する。モデルと実行binary本体は配布しない。
ページAPI呼出し/CLIプロセス/保存raw callbackを区別し、未計測の内部NN forward数と同一視しない。

Pillowを使えるPythonで、空の復元先へ実行する。

```sh
python3 docs/benchmarks/vertical-capacity/reproduce.py --output tmp/vertical-capacity-reproduced
```

元画像/crop/図形・有理数mask・全chunk座標・API/CLI境界とsourceを検査し、正式48 CSV/Markdownを
再生成してbyte照合する。文字/単語距離は別実装でも照合する。Rust/Cargoがあれば、同じコマンドに
`--rust`を付けて実際のselector126組も再コンパイル・再実行できる。ONNX featureとモデルは使用しない。

```sh
python3 -m unittest discover -s docs/benchmarks/vertical-capacity -p test_reproduce.py
```

先行仕様と制限は[容量回復](../../vertical_model_capacity_recovery.md)、
[追加対照](../../vertical_capacity_controls.md)、[回帰行列](../../vertical_capacity_regression_matrix.md)、
[証拠packet](../../vertical_capacity_evidence_packet.md)を参照。

空の復元先で、全14,839パス、正式48表・33,528 CSV行、モデル不要の再コンパイルと126判定の一致を
確認済み。結果は[verification.json](verification.json)。初回のRust再実行はCargoの作業ディレクトリが
変わるため相対入力パスを見つけられず失敗した。元のmanifestを保ち、実行用コピーの入力/callback
パスだけを復元先へ解決する修正を、Red→Green後に空ディレクトリから再確認した。
[失敗と修復履歴](reproduction-history/files.json)も残している。
標準出力/標準エラーは、末尾の空行も変更せずgzipへ保存した。履歴indexに圧縮前後のSHAを記録する。

readerの6テストを標準CIへ追加する変更は、現在のGitHub OAuth認証にworkflow scopeがないため未適用。
[変更案](ci-reader-workflow.patch.gz)を保存した。ローカルでは6件成功し、新しいRust実モデル回帰は既存CIが実行する。
