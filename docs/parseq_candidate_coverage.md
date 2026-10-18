# PARSeq候補の回復余地の測定

OCR本文の自動補正を導入する前に、固定した画像・行座標・独立した原文と
`parseq_token_candidates` APIから、文字候補に正解が残る範囲を測定する。
これは評価用example/toolであり、通常の認識経路とモデルは変更しない。

## 入出力と再現性

`export_parseq_candidates` exampleは画像、モデル、文字集合、JSON行manifest
（`id`, `bounds: [x0,y0,x1,y1]`）を受け取る。ページ全体には既存の
`normalize_chromatic_panels` を適用し、行と区間のcropは既存の
`split_line_for_input` / `crop_line_chunk_rgb_u8` と閾値220を使用し、横書き方向を
明示する。区間ごとに一回推論し、本文・候補・停止理由をJSONに保存する。
`--chunk-input-dir NEW_DIRECTORY` を指定すると、同じ認識直前RGBを可逆PNGと
対応表へ保存する。既存ディレクトリは拒否する。仕様は
[`candidate_input_capture.md`](candidate_input_capture.md)。モデル内部のtensorとは異なる。
区間連結で補った空白は固定文字として記録し、モデル候補を捏造しない。
Unicode正規化、全半角変換、本文の置換をしない。入力manifestのtruthは推論に使わない。
行検出で欠落した原文行は `bounds: null` で明示し、空の予測・候補として返す。
この行を推論対象や評価分母から無言で削除しない。

`tools/eval_candidate_coverage.py` は独立したtruth付きmanifestとexportをIDで
対応づける。ID重複・欠落・余分な行、不正な本文/文字候補の対応はエラーにする。
候補数kを指定して、以下を原文のUnicode scalar単位で集計する。

- 通常本文の厳密Levenshtein距離と、固定した最短編集経路の一致・置換・欠落・余分。
  同点の逆追跡は一致、置換、欠落、余分の順。繰り返し文字等では対応が一意でない。
- この経路の置換誤りについて、正解候補が残る順位のヒストグラムと候補外の数。
  これはこの対応に条件づけた値であり、すべての誤りの回復率ではない。
- 各出力位置を上位k文字のどれかに置換したときの最小編集距離。
  原文を参照したoracleであり、補正実装の精度ではない。位置数は固定し、EOS候補で
  文字を消したり停止後の文字を生成したりしない。全クラスsoftmaxの順位を使い、
  EOSも順位数に含めるが代替文字からは除外する。固定連結空白は置換できない。
- 数字・固有名詞を含む正読位置にも代替候補がある数。これは誤補正の危険に
  さらされる位置数であり、未実装の文脈補正の誤補正率とは呼ばない。

各行のraw本文、各区間の停止ステップ、候補と入力SHAを保存する。ページ全体の
CERには改行を含む。行単位の集計分母は改行を除くため明記する。
実行時間は共有環境での観測値に留め、速度改善を主張しない。

## 先行テスト

候補exportのUnicode範囲・区間空白・停止ステップ、評価の置換/欠落/余分、
全半角/反復/数字/名前の保持、候補数の単調性、誤った入力の拒否をRedから追加する。
実金融資料は既存の開発データであり、未知資料の汎化評価とは扱わない。

## 実行例

```sh
cargo run --features onnx --example export_parseq_candidates -- \
  --image page.png --model parseq.onnx --charset NDLmoji.yaml \
  --manifest lines.json --output candidates.json --top-k 10
python3 tools/eval_candidate_coverage.py --manifest lines.json \
  --predictions candidates.json --output report.json --k 1 3 5 10
```

manifestは `[ {"id":"line-1", "bounds":[0,0,100,30], "truth":"原文"},
{"id":"missing", "bounds":null, "truth":"欠落行"} ]` の形式。truthはexportでは使用せず、
評価toolだけが読み取る。既存データへの評価は新しい未知資料の精度を保証しない。
