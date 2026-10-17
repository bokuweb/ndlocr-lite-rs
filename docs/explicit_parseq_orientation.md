# 横書き行断片の方向を明示する

既存の自動方向判定は公式1.3.1と同じ `h * 5 > w * 4` で90度反時計回りに回転する。互換性のため既存APIの挙動を維持する。

ページ側で横書き行を検出した場合、短い断片のアスペクト比から方向を再推定すると、46×48pxの数字「１」が回転され「11」になる。モデル・画素・座標・デコードは変えず、呼び出し元が Horizontal を指定できる前処理および再利用セッションAPIを追加する。Auto は従来どおり、Horizontal は入力を回転しない。後処理・文字置換・追加推論は行わない。

検証: 既存Auto境界テストを維持し、HorizontalのBGR/NCHWピクセル一致、バッファ再利用、無効入力、固定モデルで「1」を検証する。omoの横書き行認識に採用後、金融ページ全文CERと全既存実モデル行・座標を比較する。全角/半角の誤りは本変更では解消しない。速度改善は主張せず、モデル呼び出し回数が増えないことを確認する。

呼び出し例（横書き行と判定済みの画素だけを渡す）:

```rust,ignore
use ndlocr_lite_rs::infer::parseq::ParseqOrientation;
let result = session.recognize_rgb_u8_unprocessed_with_orientation(
    rgb, width, height, ParseqOrientation::Horizontal,
)?;
```

方向が不明な入力や既存の縦書き呼び出しには、従来APIまたはAutoを使う。方向指定は呼び出し単位であり、セッションの次の認識に持ち越さない。前処理を直接行う場合は `preprocess_rgb_u8_with_orientation`、再利用バッファを使う場合は `PreprocessScratch::with_orientation` を利用する。

## 固定評価結果（2026-10-17）

先行RedはAutoで `"11"`、Horizontal指定後は `"1"`。同じセッションでAutoへ戻しても既存の `"11"` を保ち、無効入力後のHorizontal再推論も成功する。通常294件、実ONNX対象248件が成功した。

omoの候補では金融全文CERが75/473から74/473へ改善。全256行を比較して他255行の本文と全行座標が一致し、描画28ケースも変化なし。実ONNX/表123件・通常/表66件と両構成の厳格Clippyが成功。生本文・座標・モデル/真値SHAはomoの `docs/benchmarks/ocr-2026-10-17-horizontal-orientation/` を参照する。
