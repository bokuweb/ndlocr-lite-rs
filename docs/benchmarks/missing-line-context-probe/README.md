# 全文枠のない行を調べる（製品変更なし）

先行仕様は `docs/missing_line_context_probe.md`。ページの比較元はbae38d3。
前段でOCR前に固定した反復160＋帯復旧24の184画像、30/100モデルのeval/default、
736本文組を調べた。原画像と帯cropの1368候補を全件記録し、全文画線を含む1232候補だけを
none/width/strictの全文真値と比較した。部分cropに全文真値を割り当てていない。
同じ画像の複数crop/モデルであり、独立資料数や製品精度改善とは数えない。

## 見つかったこと

- 既存出力を全体候補へ一律置換すると150文字指標が退行する。
- native入力比率内かつconfidence>=.9でも、928候補の比較で54文字指標が退行する。
- native比率内で既存平均以上のconfidenceでも、744候補の比較で24文字指標が退行する。

例として、既存出力が正読した12個の0を、帯の高confidence=.986候補へ変えると12文字誤りになる。
別の混合英数字では、全文画線が既存枠に収まっているのに、.847→.853のconfidence選択で
文字距離が5→6へ悪化する。native容量やconfidenceだけを使う一律変更は採用しない。

一方、全文枠がない混合A1のJPEGでは、30モデルの既定出力は断片5行で13誤り、
同じモデルの原画像候補は4誤り。100モデルも9→4となる。
原画像候補の100 confidenceは.893で、固定.9ではこの改善も除外する。
既存断片の平均.737と比較する追加層を、仕様追記後に全候補で調べた。

全文被覆/native内/帯1本/非空既存出力/未被覆画線あり/既存平均以上という追加層では、
86候補のうち生文字距離38組が改善し、none/width/strictの258文字・単語比較は非悪化。
原画像と帯をそれぞれ独立に採点しており、真値で候補を選んだ結果ではない。
ただし、これは184画像での診断で、全287画像やCLIでは未検証。製品には接続していない。
未被覆量にはJPEGの微小な画線も含むため、1画素で復旧を決めることの妥当性も未確認。

## 検証

未被覆画線は半開bboxの和集合を使い、重なり・重複の二重計上を避ける。
境界・閾値・無効RGB・全角ASCIIとstrictの区別・部分crop非採点・相対confidenceを6テストで検証した。
80個の固定seedによる移動/拡大/box順序の性質も含む。Red→Greenと途中の誤ったstrict期待値も保存した。

全342cropは同じRust decoderで元入力を読み、bbox内のRGBとbyte一致した。
JPEGをPillowで再decodeしてRustの画素と比較する方法は使っていない。
別の一時rootへ前段archiveを展開して診断を再実行し、全1368候補を含むJSONがbyte一致した。
この再実行はモデルを呼ばず、元の一時画像パスにも依存しない。
モデル/charsetの実ファイルSHA確認は `--verify-model-assets` で追加でき、初回実行で確認済み。

前段bae38d3の全5CI（Rust3OS・実モデル2OS）は成功した。今回の製品コード差分はない。

## 再現

repo root、PythonとPillowが必要。生成時のfontやONNX実行は不要。
`--page-source-commit` の全製品source SHAをGit内の当該commitと照合する。
元画像の一時パスを辿らず、追加160画像は固定fixture ZIP、真値とcropは検証済みarchiveから読む。

```sh
python3 docs/benchmarks/visible-repetitions/unpack_evidence.py --output tmp/context-probe-replay --diagnostics
python3 tools/missing_line_context_probe.py --frozen-root tmp/context-probe-replay --page-source-commit bae38d3 --output tmp/context-probe.json
python3 docs/benchmarks/missing-line-context-probe/summarize.py tmp/context-probe.json --output tmp/context-probe-audit.json
```

`evidence.zip/.json` は全候補、初回固定floorの結果、全退行、Red/Green、Rust画素照合、
CI snapshotと診断sourceをmember SHA付きで保存する。前段2archiveのSHAも参照する。
`summary.json` は全文候補を省いた集計・参照SHAと別root再実行の一致を持つ。

次は検出枠の外に画線がある場合の全体文脈確認を実モデルで全287画像・4 API設定とCLIへ広げ、
候補の採否・追加推論負荷を別仕様/失敗テストから検証する。
