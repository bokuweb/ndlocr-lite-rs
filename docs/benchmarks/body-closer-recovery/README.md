# 本文の整列を維持した低い閉じ括弧の製品回復

先行仕様は `docs/body_closer_recovery.md`。比較元9f6a5e2（製品の動作は02ad642と同じ）。
`closer_geometry_probe` の固定1239画像と、最初のOCR前に固定した境界32画像を使う。
既知fontの合成開発対照で、未知実資料holdoutや1271の独立文書とは数えない。

画像付き単行の従来支持を維持し、本文が2成分以上・共通高さ3/4、低いcloserが本文高さ1/2以上で
交差3/4を満たす場合にも元fragmentを保持する。x区間重複・数の不一致・空白・方向/RGB不一致では保持しない。
同じ抽出を従来支持と共用し、ページfragmentには従来の厳密な高さ条件を使い続ける。
追加モデル・forward・文字の推測置換はなく、RGB/confidence・text-only/raw/候補APIを保持する。

## 全件の結果

単行1271×3モデル×3正規化=11439と、境界Cascade32×3開始bucket×3正規化=288、
合計11727の文字指標・11727の単語指標が全て非悪化。

| 経路 | 入力数 | 完全一致改善 | byte同一 | 生文字距離同値変更 |
| --- | ---: | ---: | ---: | ---: |
| 単行30 | 1271 | 28 | 1233 | 10 |
| 単行50 | 1271 | 15 | 1253 | 3 |
| 単行100 | 1271 | 21 | 1236 | 14 |
| Cascade開始3 | 32 | 0 | 32 | 0 |
| Cascade開始2 | 32 | 0 | 32 | 0 |
| Cascade開始1 | 32 | 0 | 32 | 0 |

3909本文組は64完全一致改善・3818byte同一・27同値変更。同値変更は改善に数えない。
改善は旧1239画像の64組（さらに前の未回復80組中44と、前段追加対照20）で、境界32は同一。

before/afterの単行API各3813とCascade各96を全て新規実行した。
既存3717 rawは前段archiveのSHAを検証して再利用し、新境界96 rawもbeforeから記録した。
単行beforeは全baseline、afterは全prototype候補とbyte一致。全3813のconfidenceは保存rawと前後bit一致。
全fragmentと従来の文字数支持も保存値と一致し、新body方針は独立した凍結referenceと全件一致した。

CLIは変更した全91組・境界96組・不採用の空白なし案の全退行例と系統的対照、合計257組を前後とも新規実行し、
API本文と全byte一致。終端LFを厳密に1個だけ除き、前後空白やUnicodeをstripしない。
比較元CLIは02ad642でコンパイルした凍結binaryで、9f6a5e2との製品差がないことと今回の全照合を根拠に区別する。

## Cascadeの実観測と限界

最初のAPI captureのselected_modelはnative字数からの再構成値であり、観測値と区別する。
隔離したbefore/after sourceで各pool結果受取と再認識結果受取にtest-only観測を加え、前後各96を再実行した。
getter未実装で0/96となるRedを確認後、実model_historyを記録して元のCascade本文/confidenceとも全bit一致した。
観測source/patch/binary SHAを保存し、製品のsourceに観測hookは含めない。

前後とも[30]が19、[30,50]が13、[50]が32、[100]が32で、履歴変更は0。
新たに復旧したcloserが25/45字をまたいで再認識する実画像の精度を確認したとは主張しない。
純粋画線の23/24/43/44文字では旧字数から+1へ復旧することを別に検査した。

境界のraw正読24組も未回復のため、対象範囲の353組中64回復・289未回復。
内訳は成分数240・x重複47・eligible除外2。接触/複数成分、単語空白や文字種の診断を継続する。
前段の空白なし案の6文字・32単語退行等は親benchmarkに保存し、今回真値で方針を選択していない。

## 検証と再現

- 純粋な低いcloserのRedと実モデルの `資料」` → `資料` の公開API Redを先行しGreenにした。
  prefix/internal/suffix、本文ずれ・小さい/離れたcloser・数・空白・方向・RGB、PBT64例を検査する。
- 通常は全586+追加境界1の587件、ONNXは全539+追加境界1+再生1の541件、example18件が成功。
  root環境変数なしでno-opとなる診断もONNX件数に含む。公開stateless/Session/Pool/batch/単一Session batch/cascade、
  raw/候補・confidence/RGB・不正入力後の再利用を実モデルで検査した。
- fmt/diff/対象Clippy成功。新規警告0、本体の既存18、lib testの既存28（18重複）等は全ログに保存する。
  直前9f6a5e2の全5CIも成功した。
- Page/cached/parseq/app callerのsourceはbeforeとbyte同一。共通の従来支持を全3813で再生し、既存実ページ回帰も成功。
  今回ページ287×6を全件再実行したという主張ではない。

`evaluation-manifest.json` に入力/真値/モデル/charset、実行前にコピーしたbefore/after binaryと
source/driver SHA、元forwardとportable検証の区別、実dispatch計測の根拠を保存する。
依存のCargo.lockもdispatch用snapshotに保存。モデル/binary本体は配布しない。
`text-evidence.zip` は前後本文・truth・全CLI stdout/stderrと実dispatch記録、`metrics.zip` は18 CSV/MD。
`diagnostic-evidence.zip` は原raw/入力画像・成分/不採用案参照・全Red/Green・API結果・source/script・ログ。
各JSONはarchiveと全memberのSHAを持ち、展開で全件検証する。

```sh
python3 docs/benchmarks/body-closer-recovery/unpack_evidence.py --output tmp/body-closer-restored --diagnostics
NDLOCR_BODY_EVAL_ROOT=tmp/body-closer-restored python3 docs/benchmarks/body-closer-recovery/evaluate.py
```

モデル/元画像/追加packageを置かない別rootで、同梱の採点toolを使った全18 CSV/MD＋監査のbyte一致を確認した。
凍結portable binaryでも全3813の現在製品選択/reference/fragment/従来支持/confidenceと境界成分が一致した。
未知資料の汎化・速度改善は主張しない。目的全体は継続する。
