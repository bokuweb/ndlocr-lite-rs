# 全幅/細分割/実モデル幅の文脈比較

先行仕様 `../../page_chunk_probe.md`。既定pipelineは変更していない。
固定20画像×DEIM/bandsの40ページ・730cropを対象に3案を診断した。

| 案 | 比較元 | 非悪化ゲートに失敗した文字指標 /120 |
| --- | --- | ---: |
| 既存の細かい安全chunk | 固定100文字モデル全幅 | 63 |
| 実入力幅まで隣接chunkを結合 | 固定100文字モデル全幅 | 30 |
| 全行を100文字モデルに置換 | 既定cascade | 15 |

改善もあるが各案に画像別の退行があり、一律の既定化を棄却した。
none/width/strictを別々に判定し、合計や一つの指標で悪化を相殺しない。

モデル入力比率を超える410領域中402領域を分割できた。
細分割は2168chunk、実モデル幅の結合は804chunkへ推論した。
原の各cropも730回ずつ認識しており、どちらも追加の認識負荷がある。
診断の経過時間はログにあるが、速度比較としては設計していない。

固定100文字モデルの比較元40本文は、保存binaryで実際のCLIを
`--enable-cascade false --split-long-lines false` として再現し、全件byte一致した。
元cropの全文/confidenceも2回の診断で730件とも一致。
初回で保存しなかったsanitize前の本文だけ、2回目の同一cropのraw結果から
照合して補った。原report SHAと730照合の詳細はraw-baseline-provenance.json.gz、
処理はenrich_raw_baselines.py。後の再生では最初からraw_beforeを保存する。

raw/断片結果・座標・bucket・全chunk/失敗はfine-results.json.gz / native-results.json.gz。
全文はbaseline/fine/native/cli、真値はtruth、全18採点はCSV/Markdown。
入力/model/charset/binary SHAと40コマンドはmanifest.json / cli-runs.json。
通常cascadeの比較元は前コミットdde5d3aの `../url-hyphen/*/after/`。

```sh
NDLOCR_INTRA_THREADS=1 NDLOCR_PAGE_CHUNK_PROBE_DIR=tmp/page-chunk-probe \
  cargo test --features onnx --test page_chunk_probe_tests -- --include-ignored --nocapture
NDLOCR_INTRA_THREADS=1 NDLOCR_PAGE_CHUNK_PROBE_DIR=tmp/page-chunk-probe/native \
  NDLOCR_PAGE_CHUNK_GROUP_NATIVE=1 \
  cargo test --features onnx --test page_chunk_probe_tests -- --include-ignored --nocapture
cargo test --test native_chunk_group_tests
```

結合helperは診断専用。例示・境界・非連続の白余白・不正入力・列被覆PBTの
5件でRed→Greenを確認した。元20画像は既知資料と機械変形であり、未知資料の証拠ではない。
全幅100認識は最大77文字であり、既定CLIの80文字以上の長行再分割は評価範囲外。
次はこの未検証分岐を別の固定資料で調べる。

大きいraw JSONはgzipで可逆圧縮し、原の初回reportもfine-initial-results.json.gzに保存。
compressed-reports.jsonに圧縮前後のSHAを記録し、各byteの復元を検証した。
例: `gzip -dc docs/benchmarks/page-chunks/native-results.json.gz > /tmp/native-results.json`。
標準ライブラリのgzip.openでも直接読める。再梱包のスクリプトはpack_reports.py。
