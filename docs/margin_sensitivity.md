# 横書きchunkの余白感度の回帰

omo製品経路の既存比較で、1〜3列の純白余白縮小だけでもJPEG由来の正読に余分な句点が付く反例があった。非白RGB保持とモデル入力の保持を区別するため、原chunk6件をlossless fixtureとして固定する。

先に `tests/margin_sensitivity_tests.rs` を追加し、fixture不在のRedを保存する。元製品経路の全calls/座標/本文/信頼度を再現して取得した後、原寸・RGB・縮小寸法・BGR/NCHW前処理の差を検査する。実モデルの正解改善や本番採用を示すテストではない。

モデル比較は `examples/probe_margin_sensitivity.rs` で原画像/縮小画像それぞれに純白0〜4列を右追加した全60変種を同一sessionで2回認識する。全文・信頼度の完全再現と全結果を保存する。条件の選択や句読点削除は行わない。既定APIは変更しない。

```sh
python3 tools/fetch_upstream_assets.py models
NDLOCR_INTRA_THREADS=1 cargo run --locked --features onnx --example probe_margin_sensitivity -- \
  models/parseq-ndl-24x256-30-tiny-189epoch-tegaki3-r8data-202604.onnx \
  models/NDLmoji.yaml tests/fixtures/margin_sensitivity margin-sensitivity.json
```

CIはUbuntu/macOSでも同じ120推論を実行する。検証するのは同じ環境・同じsession内の再現性であり、異なるCPU間の信頼度完全一致や精度向上ではない。出力には各条件の本文・平均信頼度・推論時間、元tensorとの差の要素数・最大値・RMSを含む。時間はdev buildの単発記録で、製品速度の比較には使わない。

ローカルの固定モデル比較では全120出力の本文/信頼度が反復一致した。原画像の誤りは2/77字、縮小だけでも2/77字。右へ2列の白を追加した正読対照 `and culture.` は `Cand culture.`、3列追加した `about Japan. We` は `Labout Japan. We` に変わった。原画像と縮小の信頼度比較では0誤りになるが、この既知6件に対する事後分析であり採用条件ではない。製品pin・既定経路は維持する。

全60出力、独立採点/公開scorerの照合、モデル/入力hash、先行Red/Greenは [omo #79](https://github.com/jlsi/omo/pull/79) の `docs/benchmarks/ocr-2026-10-18-margin-sensitivity/` に保存する。次は原画像/縮小の信頼度選択を先に固定し、独立資料・日本語・JPEG・正読対照の全件非悪化を評価する。画像別の条件調整は行わない。
