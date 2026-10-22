# 文字内部の分離成分と閉じ括弧・空白の支持

仕様は `docs/contained_closer_components.md`。比較元ed453b9。
`資料)` の料は、大きい成分のbbox内に独立した画線があるため、3字を4成分と数えて `資料` に崩れていた。
元のRGB/8近傍/area閾値は維持し、他bboxの4辺すべての厳密な内側にある成分だけを除いたcloser用viewを作る。
同bbox、端の一致、部分重複、上側dotをまとめず、包含でviewが変わらない場合は元出力を維持する。
ページfragmentと反復の従来count gateも元の全成分を使い続ける。

## 棄却案と採用条件

| 案 | 画像数 | 文字指標退行 | 単語指標退行 | 判断 |
| --- | ---: | ---: | ---: | --- |
| 包含viewだけ | 2231 | 0 | 64 | 棄却 |
| Unicode幅2のピッチ支持を追加 | 2551 | 16 | 108 | 棄却 |
| view変化時だけ・全角/混合セルの幅と間隔を確認 | 2871 | 0 | 0 | 製品検証 |

括弧だけ回復すると、印字 `」 回路図` にraw `」回路図` が出た場合に単語が悪化する。
第2案は、包含のない既存正読 `」資料` の空白まで再解釈していた。
各案の出力を読む前に新fixtureを固定し、後からtruthや画像を変更していない。
第1案352、第2案320、第3案320の追加画像は既知fontの合成開発対照で、未知実資料holdoutではない。

採用案は包含で成分数が変わった場合だけ使う。字数/x非重複・本文共通高さ3/4・closer高さ1/2/交差3/4を確認し、
元のcloserと本文を保持する。Unicode幅2の本文が2字以上ある場合は、幅1/2の混合も含めたbboxのセル幅とpitchを確認する。
幅は固定した `unicode-width 0.2.2` の `UnicodeWidthChar::width()`、セル幅は幅2の最大bbox幅＋本文最大gap。
先頭の全角1セル付近を空白と見なさず、狭いmark＋gap、または全角セルを明確に超えるadvanceだけを空白として支持する。
細い実括弧に対してこのセル支持内では幅>=ceil(本文字高/5)を要求する。既存の一般空白支持の字高/4条件は維持する。
新runtime依存はUnicode表示幅だけで、追加モデル/forwardや語句・closer種類の推測置換はない。

## 全件の結果

2871画像×3モデル×3正規化=25839の文字指標と25839の単語指標が全て非悪化。
8613本文組は727byte変更・7886byte同一。変更727のうち664はいずれかの指標/正規化で改善、63は全指標同値で改善に数えない。
未正規化の文字距離では524改善・203同値変更。完全一致へ470組改善した。

| モデル | 本文組 | 生文字距離改善 | 生距離同値変更 | 完全一致改善 | byte同一 |
| --- | ---: | ---: | ---: | ---: | ---: |
| 30 | 2871 | 220 | 73 | 200 | 2578 |
| 50 | 2871 | 130 | 44 | 117 | 2697 |
| 100 | 2871 | 174 | 86 | 153 | 2611 |

完全一致470のうち359はraw正読の後処理損失を回復し、111はモデルが省いたASCII空白だけを補って完全一致になった。
既存1879の未回復335は299へ減少し、36組を回復した。変更は既存literal69・最初の追加352内373・次の320内160・最後の320内125。
新対照込みのraw正読損失は930→571で、包含でまとまらない部品・接触・字種の問題などは残る。目的全体は継続する。

## 実行と検証

既存5637 rawは親archiveの全SHA、before/fragmentと実API照合で再利用。
追加992画像はraw/processed各2976 forwardを新規捕捉し、afterは全8613の公開Sessionを新規実行した。
全本文が候補とbyte一致、confidenceが元rawとbit一致。original componentのbbox/area、従来count gate、fragmentも全8613で保存値と一致した。
全変更と新320対照の和集合570画像×3モデルの1710 CLIを前後とも新規実行し、全APIと一致した。
終端LFを厳密に1個だけ除き、元の空白をstripしない。

純粋画線と公開Sessionの `資料)` → `資料` のRedを先行してGreenにした。
4辺/同bbox/部分重複/dot/入れ子・セル幅/gap/advanceの閉じた境界と4種類のPBT（各64例）を検査する。
3モデルのstateless/Session/Pool/batch/単一Session batch・raw/候補/fragment・RGB/confidence・エラー再利用・Cascade開始3/2/1も検査した。
通常616件、ONNX CI対象519件と明示rootの8613再生1件、example18件が成功。
環境変数なしでno-opとなる診断もCI件数に含む。fmt/diff/対象Clippy成功、新規警告0（既存本体18・lib test28/18重複）。
ed453b9の全5CIも成功した。

保存したnative字数からは25/45字の新しい閾値跨ぎ0・字数減少0。これは実dispatch観測ログと区別する。
公開Cascadeの短い正負例は実行したが、今回の復元で再認識先が変わる実画像の精度は未検証。
全ページの新規再測定・速度・未知資料汎化も主張しない。以前の未検出行復旧で記録した単語12指標退行は別baselineの課題で、最新実装での再評価は次段とする。

## 証拠と再現

- `images.zip`：原画像・全decoded RGB PNG・3段階のfixtureとgenerator/manifest。
- `diagnostics.zip`：全方針本文を含むraw/成分/確率/停止理由JSON、API/CLI結果index、各段階の実行前凍結source/binary/Cargo/model/input SHA、全ログ。
- `text.zip`：全truth、実CLI body/stdout/stderr、成功案と棄却2案の54表と監査。

同名JSONが各archiveと全memberのSHAを持つ。モデル/binary本体は配布しない。
方針別TXTはJSONから生成し、重複を省く。

```sh
python3 docs/benchmarks/contained-closer-components/reproduce.py --output tmp/contained-closer-restored
```

同梱の正式採点toolで成功/棄却案の54表と3監査をbyte再現する。
Rustでのモデル不要再生は `NDLOCR_CONTAINED_CLOSER_VERIFY_ROOT` と `NDLOCR_CONTAINED_CLOSER_OFFLINE=1` を与え、
凍結製品executableの `verify_contained_closer_actual_api_and_replay` を明示実行する。
診断のbefore captureはed453b9のsourceを前提とし、現在の製品へ直接適用する前提ではない。

別rootで23819 memberのSHA、54表＋3監査のbyte一致と、同じ製品executableによる8613件の原bbox/area・従来支持・fragment・製品選択の再生一致を確認済み。
