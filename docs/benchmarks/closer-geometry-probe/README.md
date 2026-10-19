# 閉じ括弧の高さと本文の整列を分ける診断

先行仕様は `docs/closer_geometry_probe.md`。比較元02ad642。
OCR前に80文字列×2字体×20/40px×2形式の640画像と、文字なし8図形×2形式の16画像を固定した。
既存583と全1239画像、30/50/100の3717本文組を比較する。
新しい丸ゴシックと明朝の合成開発対照で、未知実資料holdoutではない。

製品関数とcallerは変更しない。`visible_repetitions.rs` の追加は `cfg(test)` の診断moduleだけ。
同じrawを現在の製品方針・fragment・空白なし一律保持・高さ緩和・本文整列を維持する案へ渡す。
同じRGBから独立成分を得て、モデル確率や真値を候補選択へ使わない。

## 全件の結果

各案について1239×3×none/width/strict=11151の文字・単語指標を正式toolで採点する。

| 案 | 生本文の変更 | raw正読の完全一致回復 | 元の未回復80組の回復 | 文字退行指標 | 単語退行指標 |
| --- | ---: | ---: | ---: | ---: | ---: |
| 一律fragment保持 | 952 | 329 | 80 | 213 | 338 |
| 空白なし一律保持 | 596 | 327 | 80 | 6 | 32 |
| 最小高さに対する共通区間 | 92 | 64 | 44 | 0 | 0 |
| 本文の整列を維持 | 91 | 64 | 44 | 0 | 0 |

本文の整列を維持する案は64完全一致改善・3626byte同一・27生文字距離同値変更。
正規化を含む改善指標は192。同値変更は改善に数えない。
高さ緩和だけの案は同じ64を回復するが、`【123456】` のraw `【123456)` の同値変更を1組余分に出す。
本文整列を維持する案を次の製品/API検証対象とする。これは製品での精度改善の完了報告ではない。

原rawの正読を現在の処理が崩す329組の残り265組は、成分数216、x区間重複47、eligible除外2。
元の80組中36も未回復。複数成分・接触・空白の候補は引き続き扱う。
空白なし一律保持の反例には `情）報` → raw `報告)報`、`情報）` → raw `情報){` がある。
文字数と画線の対応を確かめずに保持すると、空白なしでも退行する。

## 検証と証拠

- 通常583件、ONNX lib263件が成功。ONNX libの環境変数なしの診断はno-opとなるものを含む。
  図形を含む656 fixtureの全組合せ、純粋Red/Green・境界・負例、拡大/移動/位置のPBT64例を検査した。
- 既存1749 rawは前段のarchive/model/入力/source SHAを確認して再利用し、現在baselineが前段の全CLI本文とbyte一致した。
  新656×3=1968 forwardは現在のSession raw候補APIを1回ずつ呼び、全step上位2確率・停止理由を記録した。
- 元RGBをRustでPNG保存して再decodeした全画素を比較した。全3717で診断の成分抽出と既存の文字数支持も一致。
- `chunks_exact` の診断内の新規Clippy警告を修正し、前後で全3717の成分枠・baseline・全候補が一致した。
  model-free再生7テスト（純粋6+再生1）も成功。対象Clippy成功、新規ファイルの警告0。
  他の既存の本体18・lib test28（18重複を含む）等の警告は記録する。
- 直前02ad642の全5CIが成功した。

`evaluation-manifest.json` は新旧入力/モデル/charset・binary・source/driver/spec SHAと再利用元を持つ。
捕捉前のmanifest/specとdriver/visible sourceの完全なbyteも保存し、記録SHAに一致する。
元forwardの凍結binaryと、後のmodel-free検証binaryを区別する。binary/モデル本体は配布しない。

`text-evidence.zip` は6方針の全3717本文とtruth、`metrics.zip` は9 CSV/MD。
`diagnostic-evidence.zip` は全raw/RGB・連結成分・確率・停止理由・Red/Green・source/script・ログ。
各JSONはarchiveと全memberのSHAを持つ。

```sh
python3 docs/benchmarks/closer-geometry-probe/unpack_evidence.py --output tmp/closer-geometry-restored --diagnostics
NDLOCR_GEOMETRY_EVAL_ROOT=tmp/closer-geometry-restored python3 docs/benchmarks/closer-geometry-probe/evaluate.py
```

再採点にはモデル・元画像・追加packageは不要。保存した全6方針本文から9 CSV/MDと監査を再構成する。
別rootでの全byte一致と、凍結したportable test binaryによる3717 raw/RGBの成分・全方針選択の一致を検査する。
未知資料の汎化・速度や、製品の全CLI/APIでの回復をこの診断から主張しない。
