# 下側の孤立小画線を本文の文脈へ戻す

実装前仕様: `docs/dense_crop_probe.md` / `docs/isolated_trailing_mark.md`。
比較元はe9b1bd3。先行79画像と、モデル診断を読む前に固定した新24画像の全103画像を使用する。
新24は独自和文/英文2種、細いW2/太いW8、20/40px、上下余白0/6pxとJPEG75。
画像の数は独立文書数ではない。既知資料・合成・機械変形の開発回帰として扱う。

## 製品経路の全件比較

CLIは既定cascadeのDEIM設定/bands設定。PageRecognizerは30/100それぞれ、
parallelism=1、DEIM threshold=.25、line confidence=.3、drop empty=true。
同じモデル・真値・辞書なしの設定で、全103×4×3=1236画像別文字距離が非悪化。

| 経路 | 本文改善 / 103 | byte同一 / 103 |
| --- | ---: | ---: |
| CLI DEIM設定 | 3 | 100 |
| CLI bands設定 | 7 | 96 |
| PageRecognizer30 | 0 | 103 |
| PageRecognizer100 | 0 | 103 |

10本文組はnone/width/strict全てで文字距離が改善し、他402本文組はbyte同一。
既存20画像のCLI/runtime計80本文も同一。runtimeはbbox/confidenceも全103画像で同一。
4経路を別々に集計し、CLIの改善をruntimeの改善として数えない。

| 例・経路 | 生文字誤り |
| --- | ---: |
| japanese_gothic36 / bands | 117 → 0 / 120字 |
| japanese_mincho36 / bands | 120 → 0 / 120字 |
| dense_library_bold36_original / bands・DEIM設定 | 各119 → 0 / 120字 |
| dense_library_bold36_jpeg75 / bands・DEIM設定 | 各119 → 0 / 120字 |
| dense_library_maru36_jpeg75 / bands | 120 → 0 / 120字 |
| dense_library_maru36_original / bands | 119 → 1 / 120字 |
| crop_ja_heavy40_pad6 / bands・DEIM設定 | 各135 → 4 / 135字 |

**単語指標は追加で2本文組（同じ新画像×CLI 2設定）のnone/width、計4指標が悪化する。**
新しい太字和文の空白区切り単語誤りは各1→2。`word-regressions.json` と全CSV/MDを保存する。
文字距離の先行採用ゲートと分離し、空白を消す後補正は追加しない。

DEIM設定の改善3組はいずれも生DEIMに有効行がなく、CLIが帯検出へfallbackした結果。
PageRecognizerには同じfallbackがなく、これらは前後とも空。旧2画像はinventory、
新1画像はmissing-deim.jsonで有効DEIM cropが0と確認した。これが次の未解決対象である。

## 原因分離と棄却した上下余白

元120文字の36px和文2例は、bands cropで読点だけが幅25pxのchunkとなった。
生100モデルは `...` / `1.` と誤読し、全chunk高confidence条件を満たせなかった。
前後の文脈へまとめた診断は、30/100×2字体×左右の全8条件で120文字が完全一致。
製品では文字内容を見ず、下側の単一8連結小画線・近い字間・本文の支持・実入力幅で前へ戻す。

全79画像を元画像/bands/DEIMで走査し、条件に合う36画像88cropを詳細記録。
初回16字以下の79cropで0/1/2/4/8pxの固定余白を比較し、6470 chunk＋88全幅の推論を保存した。
全入力・全選択bboxは `inventory.json`、詳細は `probe.json.gz`。
同じcropの0pxに対して、1/2/4/8pxは各12/13/9/11件の生文字距離を悪化させる。
既知正読0→3などもあり、一律余白を棄却した。これは診断chunk連結の比較で、通常CLIと混同しない。
初回の帯閾値200の準備は中断し、実CLI既定220で全件を実行し直した。未完了結果は混ぜていない。

## 証拠と検証

- `evaluation-manifest.json`: 全image/truth/model/charset、前後binary、824予測本文のSHA。
- `text-evidence.zip` / `.json`: 全前後本文・固定truth・runtime bbox/confidence・CLI stderr。全1185 memberとarchive SHAを確認して保存。大きい本文をPR差分から分け、元byteは保持する。
- `before-runs.json` / `after-runs.json`: CLI全argv/exit/所要時間。比較元158本文は前段の同一binary/image SHAを照合して再利用、新24画像48 CLIは新規。候補206 CLI、前後各206 runtimeページを実行した。
- `audit.json`: 全1236文字距離の差分30件と退行0。`totals.json`は生の全合計、全12 CSV/MDに文字/単語指標を併記。
- `probe-archive.json`: 5,470,117-byte JSONのSHAとgzip SHA。解凍byte一致を検証済み。`score_padding_probe.py`で凍結済みprobeの採点を再現できる。
- `focused.json` / `neighbor.json`: 36px2例の全余白と隣接文脈。真値は候補選択へ渡さない。
- `*-red.log` / `*-green.log`: 未実装の幾何/PBTと実モデルの失敗→成功。通常533件＋新境界2件、ONNX対象459件が成功。
- `onnx-command.json`: 全対象/環境。新方針・実30/100モデルの4画像条件はlibから既存ONNX CIで自動選択、外部CLI回帰はローカル対象に明示追加した。

fmt/diff/対象Clippy成功（変更箇所以外の既存の本体17警告＋example 1警告）。
採点の初回は並行実行中のruntime100が未完了で停止した。推論を再起動せず、
全出力が揃ってから採点だけをやり直した（`incomplete-runtime-scoring.log` / `evaluation.log`）。
比較元runtimeの既存79×2本文は保存済み結果と全てbyte一致した。

## 再採点

repo rootでPython標準ライブラリのみ。モデル再推論は不要。

```sh
python3 docs/benchmarks/isolated-trailing-mark/unpack_text_evidence.py --output tmp/isolated-mark-replay
NDLOCR_DENSE_EVAL_ROOT=tmp/isolated-mark-replay python3 docs/benchmarks/isolated-trailing-mark/evaluate.py score
python3 docs/benchmarks/isolated-trailing-mark/score_padding_probe.py
```

新規のモデル比較では前後各103画像を同じ設定で実行する。
`NDLOCR_DENSE_MANIFEST=docs/benchmarks/isolated-trailing-mark/evaluation-manifest.json`、
`NDLOCR_DENSE_RUNTIME_OUTPUT=tmp/dense-crop/after`、`NDLOCR_INTRA_THREADS=1` を指定し、
`cargo test --features onnx --test dense_page_runtime_probe_tests -- --include-ignored` でruntimeを保存。
ONNX CLIを `tmp/dense-crop/candidate-ndlocr` へコピーして `evaluate.py after` を使う。
再実行の前に `unpack_text_evidence.py` の既定出力先へ前後本文/manifestを戻す。

生probeの再推論は製品比較元e9b1bd3に診断testを置いて、
`inspect_dense_crop_and_fixed_vertical_context` を明示選択して実行する。
現在版では分割方針が異なり、同じ入力でもprobeのchunkは変わる。

追加推論を含む速度・メモリ比較は未計測で、高速化は主張しない。検出枠の欠落、未検出時のruntime空結果、
曖昧な重複、句読点の誤挿入、URL/空白残差、未知実資料への汎化は未解決。
