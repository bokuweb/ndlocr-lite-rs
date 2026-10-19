# 画像が支持する単行の閉じ括弧を保持する

先行仕様: `docs/literal_closers_probe.md` / `docs/literal_closers.md`。
比較元d6a67fe。新fixtureは最初のOCR前に80文字列×2字体×PNG/JPEG75=320画像を固定した。
前/後/内部/対応した括弧、5括弧種、単独/2個連続と10種類の対照を含む。
既存の単行263と合わせ、全583画像×30/50/100モデルをAutoで比較する。
既知フォントの合成開発対照で、未知実資料のholdoutや583の独立文書とは数えない。

## 全件の結果

全583×3×none/width/strict=5247の文字・単語指標が非悪化。
全1749 after CLIを新規実行し、保存した同じrawとRGBからの製品方針再生結果に全byte一致した。
比較元の新960 CLIは新規実行。既存789本文は前段archiveの全SHAと、今回のrawから構成した
beforeの全byteを検証して再利用した。再利用にstdoutを捏造していない。

| モデル | 生文字距離改善 | byte同一 | 距離同じで本文変更 | raw正読回復 | raw正読の未回復 |
| --- | ---: | ---: | ---: | ---: | ---: |
| 30 | 38 | 535 | 10 | 38 | 38 |
| 50 | 32 | 547 | 4 | 32 | 19 |
| 100 | 34 | 540 | 9 | 34 | 23 |

未対応閉じ括弧の削除で元のraw正読184組が崩れ、104組を回復した。残る80組は未解決。
既存263画像の全789本文は同一。文字種の同値変更を改善に数えない。
全5種の括弧を一律保持すると73文字指標・108単語指標で退行したため採用しない。
fullwidthの印字 `A）BC` にraw `A) BC` が高確率で出るなど、幅と空白の複合誤りがある。
raw捕捉後に追加した .5/.7/.9/.99 の確率閾値探索も全て退行した。
閾値gridを事前登録や独立検証とは扱わず、全案の出力・確率・採点を保存する。

今回は元のfragment方針候補が長く、空白がなく、実効Horizontalで、正しいRGB寸法と
既存の独立連結成分の文字数支持を満たす場合だけ保持する。追加forward/モデルはない。
RGB・confidence、text-only sanitizer、raw/候補API、ページfragmentの関数は変更しない。
全1749行のfragment再生も以前とbyte一致。parseq/cached/app/page callerのsource SHAは比較元と一致する。
ページ全287画像・6経路を今回新たに再実行したという主張ではない。既存の実ページ回帰は通す。

通常576件、ONNX対象526件、example18件が成功。ONNX件数には明示環境変数がないと
no-opになる診断2件も含める。公開stateless/Session/Poolとbatch/単一Session batch/cascade、
raw/候補・不正入力後の再利用・RGBとconfidenceを実モデルで検査した。
純粋Redは7画線を6字へ落とす期待値と未支持・空白・方向・RGB境界を先行する。
実モデルRedは凍結した比較元CLIの `ABC)` → `ABC` で、公開Rust APIのGreenを別途記録する。
既存の幾何PBT48例も通す。fmt/diff/対象Clippy成功（既存本体18+example1、新規警告なし）。

## 証拠と再現

- `evaluation-manifest.json`: baseline/candidate/診断binary、各source/driver/model/charset/入力/真値のSHA。
  raw捕捉binaryとCLI binaryを実行前にコピーして凍結。raw captureのsource/driverも記録SHAとbyte一致で保存した。
- `text-evidence.zip`: 全before/after本文とtruth、実行したCLIのstdout/stderr。
  本文抽出は終端LFを厳密に1個だけ除き、余分なstripをしない。全stdout roundtripを検査する。
- `metrics.zip`: 実CLI9 CSV/MDと、raw・一律保持・画素支持・4確率閾値等の診断9 CSV/MD。
  文字だけでなく単語指標も全件の正式ツール採点で照合する。
- `diagnostic-evidence.zip`: 全1749 raw・各step上位2確率/stop理由、Rustがdecodeした583 PNG、
  全不採用案の本文、Red/Green・全試験ログ・source/実行script/argv/終了コード/再利用記録。
  binary/モデル本体は配布しない。新fixtureの原PNG/JPEGとtruthはfixture ZIPにある。
- 各同名JSONはarchiveと全memberのSHAを持つ。展開時に全件検証する。

```sh
python3 docs/benchmarks/literal-closers/unpack_evidence.py --output tmp/literal-closers-restored --diagnostics
NDLOCR_LITERAL_ROOT=tmp/literal-closers-restored python3 docs/benchmarks/literal-closers/evaluate.py score
```

上の採点にモデル・原画像・追加packageは不要。全9 CSV/MDと `cli-audit.json` を元の表とbyte比較した。
診断9表も正式ツールで初回の全char/word件数に一致した。
さらに別rootの保存raw/RGBを凍結したportable replay test binaryで再生し、全1749の製品選択と
fragment本文が一致した。検証用binaryのSHA/driverは元forwardのbinaryと分けて記録する。

未回復80組の既存幾何条件の除外理由は共通高さ44、連結成分数36。
複数成分の和文、背の低い括弧、モデルの幅/空白・文字種誤読は次の診断対象。
この部分改善を目的全体の達成とは扱わず、未知実資料の汎化や速度改善は主張しない。
