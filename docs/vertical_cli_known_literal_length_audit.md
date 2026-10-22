# 既知arial121回帰のGTとliteral期待値

`tests/fixtures/vertical_fallback_page/case.json`は、作成時からGT121 scalarと
既存FVBのliteral出力120 scalarを分けていた。GT中のU+3000一字の欠落は
`known_raw_error`に明記されており、`docs/vertical_page_supported_page_regression.md`にも記録済み。
候補FVCのknown CLI回帰が一致するのは、この120字literalである。
会話中の「121字本文が一致」という表記を訂正する。画像/GTを121字例と呼ぶことと、
候補のraw出力が121字すべて正しいことは異なる。

Rootへの接続準備ではfresh FVB CLIがDEIM off/onとも360 scalarの誤った分断出力となり、
同じ新テストがliteral不一致の2 Redとなった。CLI自体は両方exit0で、stdoutは実body+LFだった。
凍結v2 CLIへ同じtest binaryを向けたknown precheckは120字literal/末尾LFの2 Green。
これは本体への接続完了でも新規改善の追加計上でもない。

Root before readerで、GT121字、literal120字、U+3000だけを除くと完全一致すること、
それぞれの元SHAを検証する。reader初回の121字assert失敗も保持し、期待値とGTを修正しない。
全件/新72の公式採点は元GTで計算しているため、この字数表記の訂正による採点変更はない。
