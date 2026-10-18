# 四辺成分回復の棄却記録

先行仕様: `../../deim_component_probe.md`。
通常cropと既存明示API（閾値220、最大24px）を固定20画像で比較。
368領域→219変化、50/100両モデル438対の生結果を省略せず保存した。
モデル単体のcrop診断であり、製品CLIのCER比較ではない。

```sh
NDLOCR_INTRA_THREADS=1 NDLOCR_COMPONENT_PROBE_DIR=tmp/component-probe \
  cargo test --features onnx --test deim_component_probe_tests -- --include-ignored --nocapture
```

入力画像は原fixture5件と `../deim-edge/quality/images/` の固定15変形。
SHAはmanifest.json。既定pipelineに四辺成分回復は導入していない。
