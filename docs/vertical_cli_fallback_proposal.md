# FVC: CLIの未検出単縦列にも支持付き方向fallbackを使う（実装・新対照・NN前）

FVBはPageRecognizerだけの隔離案で、CLIは別のCachedPageRecognizer/bands経路を使う。
原FVB全件比較のPage30/50は改善したが、CLIの変更は意図していない。
したがってCLIへの同じ構造的回復は別案FVCとして検証する。FVBを採用済みとは扱わず、
FVBの全件gateを整理するまでFVCの実モデル認識や本体接続へ進まない。

## 対象と保持

- 対象は既存CLIがbandsへ進む場合だけ（DEIM無効、line_*検出が空、既存の検出失敗fallback）。
  有効な検出がある場合は追加認識0、旧経路を保つ。検出の扱いをこの案で広げない。
- 背景処理後の同じRGBに対し、FVBの単列・縦横比・幅支持・anchor数・文字らしい画線を再利用する。
  出力は非空白数>=ceil(anchor/2)、4種類以上alnum、finite0.9..1と利用者のmin confidenceを必要とする。
  文書種/語句/GTへ依存した訂正、支持条件やconfidence閾値の調整はしない。
- 既存CachedPageRecognizerを一度だけloadする。callbackは既存initial batch/cascadeを使い、
  enable_cascade、存在するモデル、30→50/50→100閾値を保持する。
  縦列の文字数bucketだけは読み方向のheight/widthを既存estimatorへ渡す。
  新たにモデルを探索/ダウンロードせず、再帰pool lockも作らない。
- 採用結果を再認識せず、元boundsと結果を後段へ一度だけ渡す。不採用/失敗なら
  legacy bandsから認識まで従来どおり行う。dict/rule-pack/構造規則/読み順の位置を変えない。
  方向支持はPageと同じ220を使い、legacy bandsのbinarize_thresholdは利用者の値を保持する。
- line_crop_paddingは元RGB上の同じ外側空白を既存のclamped bbox拡張で追加し、
  NNへ渡した範囲と公開bboxを一致させる。0の挙動とPageの既定動作は変えない。
- Page/native inferenceとH/Oの数値・採否は変えない。共有helperの可視性/引数を変更するなら
  Pageの既定引数で同じcrop RGBと採否になることを先にテストし、source対応を残す。

## Test firstと固定対照

仕様→純粋callbackテストRed→Green→隔離CLI接続の順で小さく作る。
検出有りno-op、失敗後legacy、採用結果再使用、元RGB/境界/余白、min confidence、
model/cascade設定と不正入力後再利用、default/feature-disabledを明示する。
モデルmandatoryの実CLI Red/Greenは、既知FVB fixtureを使う。CLI stdout/bodyの末尾LF framingも保持する。

追加NN前に新72を固定する。既知3font・size45/advance53・余白4/16で、新しい日本語段落の
43/63/83/103/123/143 scalarを36本文へ描画する。別12反復本文、12縦長の横書き行群、
12非文字図形（3背景×2形状×2密度）を加える。PNG/RGB/GT/非重複とglyph bboxを独立確認する。
known fontの合成開発対照であり、未知資料や学習未重複を主張しない。

新72のCLI DEIM/bands×cascade on/offの4経路を、FVB基準と隔離FVCで前後測定する。
公式none/width/strictと独立距離を全件照合し、入力別文字/語の悪化0と新本文のraw文字改善/語非悪化を必要とする。
無改善/悪化なら全案を保留する。先行成功後も、全既知入力のCLI4経路、Page6経路の不変、
line padding/min confidence/binarize threshold/quality boostなどの設定対照、必要suite/CIを別に必要とする。
追加実資料は別の事前固定runとし、進行中のFVB比較へ途中で混ぜない。

source/Cargo/model/charset/input/GTとfreshな異なる実行物を固定する。failed/partialも保存し、
画像結果/CLI process/load/内部forwardを区別する。未計測値はnull。本体へは検証後に接続する。
