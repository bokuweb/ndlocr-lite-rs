# FVB画素走査のiterator整理（実装前）

本体e559のONNX Clippyは19警告で、追加したnonwhite_envelopeのchunks_exact(3)が1件増えていた。
新しい同種警告を既存18件へ戻すため、同じ完全RGB triplet列をas_chunks::<3>().0.iter()で走査する。
判定閾値・BBox拡張・NN callback・本文処理は変更しない。remainderは両方式で走査しない。

先にランダムな寸法とRGB点を持つ有効bufferを、独立した座標ごとの期待BBoxと照合するPBT128例を足す。
既存実装でGreenを確認した後にiteratorだけを置換し、同じtest/既存13testを再確認する。
ONNX Clippyで新しいwarningが消えたことを確認する。これはrefactorであり、新たな精度改善とは数えない。
追加実資料48結果はe559のsource/実行物で完了済みとして凍結し、そのデータを書き換えない。

all-targets Clippyは成功したが、既存のFVB境界テストの`1 * w`にも恒等演算警告があった。
同じ座標`(w + 1) * 3`へ簡約する。製品の推論動作やテスト対象の画素は変えない。

最終確認: 通常lib440成功/38ignored、FVB純粋14成功、fmt成功。ONNX all-targets Clippyも成功。
lib18警告、lib-test30警告（17重複込み）を区別し、追加したFVB二ファイルのwarningは0だった。
raw記録は`docs/benchmarks/vertical-page-fallback/iterator-refactor-final-QA-v1.*`へ保存した。
