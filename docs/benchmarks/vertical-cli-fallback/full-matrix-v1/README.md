# Frozen FVC complete matrix, before full NN

The separate new72 gate passed: 576 results, 1,728 verified CSV rows, 30 raw gain
routes over 14 body images, and zero input-level character/word regressions.
This full packet does not assert adoption or a completed full gate.

`bundle.json` authenticates `extra-data-v1.zip` (318 physical PNGs and 323 GT aliases)
and references the immutable original FVB bundle and FVC adapter source packet.
The complete union is **1,239 physical IDs / 1,245 scoring IDs / 1,238 unique RGB**.
It includes the separately fixed C1 new96 as inputs; C1 inference is excluded.
The existing duplicate RGB and every ambiguous GT alias remain separate IDs.

`plan.json` specifies seven kinds × eight complete shards. Page kinds include
30/50/100 with eval/default, and CLI kinds cover DEIM/bands × cascade on/off.
Both variants produce 24,780 image results and 74,700 official metric rows.
All input-level character/word counts must be nonregressing; all Page body/bbox/
confidence bits must be identical. The 30 gains are already known, not new again.

The dispatch-only Linux NN jobs build both frozen sources once with fresh own
artifacts, freeze distinct CLI hashes, Page hashes and portable ONNX runtime
libraries, then verify the same package in every worker. The original 281 FVB
candidate files match the baseline; its extra file is the real-CLI test. Harness
source and build logs travel with the package. Models are fetched by pinned SHA.

Failed and partial attempts remain separate artifacts. Selection requires one
complete current-build job per planned kind/shard; missing/duplicate/foreign data
cannot satisfy coverage. NN argv plans and images-only manifests are saved before
recognition. Internal forward/CLI session counts stay null.

```sh
python3 -m unittest discover -s tests -p test_vertical_cli_fallback_full_matrix.py
python3 tools/vertical_cli_fallback_full_matrix.py --validate
```

Full recognition is started explicitly with the workflow dispatch. Ordinary PR
changes run model-free contracts. See `protocol-before-nn.md` for the fixed scope.
No fonts, model weights or native executables are stored in Git.
