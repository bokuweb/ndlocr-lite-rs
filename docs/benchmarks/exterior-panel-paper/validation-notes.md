# 外側白紙の回帰検証

仕様とテストを先に追加し、旧revision `e4a213c55b1bfd8f7aa88be8cd9271ab4aecf182` で白紙保持・PBTの2件が失敗することを確認した。内部字形と斜め接触の2件は旧実装でも成功した。実装後は4件すべて成功。

`cargo clippy --locked --all-targets -- -D warnings` は既存の6警告（appの引数数3件、preprocessのrange loop、outputの引数数、background既存nested if）で失敗する。追加モジュール由来の診断はない。CIと同じ `cargo clippy --no-deps` を別途実行する。既存警告を今回の精度修正に混ぜて抑制しない。

実モデル・全文の精度比較はomo側の凍結研究ケースで行う。局所テストの成功はproduction readyの宣言ではない。
