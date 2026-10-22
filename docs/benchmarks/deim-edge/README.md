# DEIMの左右境界で切れた短い横画線

仕様は `../../deim_edge_glyphs.md`、基準はce7b050の挙動を再現する保存binary。
公式モデルとcharsetは `../../upstream-assets.json`。新しい文字置換はない。

- `diagnostic-red.log`→`diagnostic-green.log`: 画像に存在する「一」の17列がcrop外。
  [49,16,1054,40]→[30,16,1054,40]で50/100両モデルと既定CLIが正しい先頭を返す。
- `diagnosis-{before,after}.json` / `diagnosis-audit.json`: 生DEIMは同一。モデルの変更は先頭1字だけ。
- `crops/{before,after}.png`: 実際にモデルへ渡した元RGBのcrop。
- `structure-red.log`→`structure-green.log`: 合成/cascade/PBT 3失敗→4成功。
- `final-structure.log`: 提案同士の衝突を含む最終5件成功。
- `default-tests.log`: 通常486件成功。後から追加した境界1件は上の最終5件で検証。
- `onnx-tests.log`: 実モデルを含む395件成功。PageRecognizerも正しい先頭を保持。
- `parallel-page.log`: 回収を使うNDL資料のページ並列1件成功。
- `clippy.log`: 対象Clippy成功、既存17警告（変更箇所の新規警告なし）。

`cli/` は同梱5資料×DEIM/bands×2版の20プロセス。NDLのDEIMだけ変わり、
生60→59、width53→52、strict11→10/1058字。先頭「一」1字の保持だけで、他9組の本文はbyte同一。
基準の全10出力は前フェーズの保存出力をbyte単位で再現した。
全30画像別文字指標が非悪化。真値は `cli/truth/`、hashは `cli/truth-manifest.json`。

`quality/` は推論前に固定した同じ5資料のJPEG90/75（subsampling=2）と各辺半分の
bilinear PNG、15画像×2設定×2版の60プロセス。全90画像別文字指標が非悪化。
NDLのDEIMだけJPEG90で生62→61、JPEG75で85→83（「箱」→「籍」もモデルで改善）。
他28組の本文はbyte同一。縮小PNGでの改善はなく、独立の実スキャンや未知資料ではない。

追加画像を `quality/images/`、真値を `quality/truth/` に凍結。`quality/manifest.json` は
Pillow版・変形・元ファイルとhash・変形後hashを持つ。出典と利用条件は元の
`tests/fixtures/eval/README.md` と各metadata JSONに従う。JPEG化/縮小を行った加工画像である。
全80プロセスのargv・binary hash・stderrは各 `runs.json`。本文差分は `all-edits.json`。
矩形内のAA画素も原RGBを使うが、外側判定は閾値220の前景であり、任意の薄い画線や
JPEGにおける完全な回復を保証しない。

再現:

```sh
cargo test --test detected_edge_bar_tests
NDLOCR_INTRA_THREADS=1 \
NDLOCR_TEST_DEIM_MODEL=models/deim-s-1024x1024.onnx \
NDLOCR_TEST_CHARSET=models/NDLmoji.yaml \
cargo test --features onnx --test deim_edge_glyph_tests -- --include-ignored
python3 tools/eval_ocr_metrics.py --truth-dir docs/benchmarks/deim-edge/cli/truth \
  --system before=docs/benchmarks/deim-edge/cli/deim/before \
  --system after=docs/benchmarks/deim-edge/cli/deim/after --normalize width --require-all
```

`quality`、`bands`、`none`/`strict` も同様に再採点できる。
CI設定はworkflow scope制約のため変更せず、新規実モデル対象はローカルで実行した。
