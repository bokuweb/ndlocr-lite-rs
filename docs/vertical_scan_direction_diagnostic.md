# 単縦列の残差: 紙の背景と方向支持の診断（追加NN前）

FVC v2の全件比較を再確認したところ、一葉3列のbandsは改善したが、地獄変3列の
bandsは不変だった。DEIM経路では50/100と同じ本文になるため、モデル追加よりも
未検出時の画像方向支持が次の調査対象になる。

まず既知6列だけを追加NNなしで診断する。画像/GT/既存予測は変更しない。
元RGBの平均明度、220未満の画素、全非白envelope、長軸の連続ink run幅と
横断extent、および既存Otsuと同じ整数比較・最初の最大値によるsupport maskを記録する。
原画素をOCR用に二値化しない。Pythonによる幾何再計算はRust実行経路の採否観測とは
区別し、単独で製品改善や正確な棄却理由を断定しない。

背景を除いたsupportが成立する根拠が得られた場合だけ、別仕様・失敗テスト・
原画素保持・負例・未測定対照を先行し、背景用supportの隔離案を検討する。
既知6列を見て方向閾値やGTを調整しない。根拠がなければ診断で終了する。

## 再計算の契約

`tools/diagnose_vertical_scan_support.py` はモデル不要の診断だけに使う。
不正寸法/RGB長を拒否する。固定220では元の全非白envelopeを1px拡張・画像内clampした
短軸幅を使い、sumRGB<660の長軸runを数える。Otsuではfloor(sumRGB/3)のhistogramを
整数比で比較し、同点は最初のthreshold、threshold binは黒とする。support maskの黒の
包絡を同様にclampし、その短軸幅を使う。run幅/横断extentがともに短軸の3/5以上をanchorとする。
白紙/単色にはOtsu thresholdなし。maskは診断専用で認識へ渡さない。

白紙・灰色紙上の既知12図形・RGB平均の切捨て・threshold同点・不正入力を先行テストとする。
実資料6列は公開controls packetのhashを検証して同じ関数へ渡す。これは方向支持の一要素の
再計算であり、Rustの列検出/band支持/認識出力の採否を代替しない。

## 結果: mask単独の案は進めない

4件のhelper未実装Red→4 Green。公開controlsの元RGB全byteを検証し、追加NN0で次を得た。
初回の探索計算はenvelope外側1pxのclampを省略していたため、採用表には使わない。
公開v2は上の契約どおりclampし、fixed220は全非白envelopeを使う。

| 既知列 | fixed220 anchor | Otsu threshold | Otsu anchor |
| --- | ---: | ---: | ---: |
| 一葉1 | 10 | 164 | 3 |
| 一葉2 | 12 | 166 | 4 |
| 一葉3 | 7 | 168 | 2 |
| 地獄変1 | 1 | 162 | 0 |
| 地獄変2 | 2 | 162 | 3 |
| 地獄変3 | 1 | 162 | 1 |

地獄変の固定220のrunは1〜2個で、紙の明度が閾値より暗くつながっている。
Otsuでrunを分離できても、この短軸幅に対するanchorは0/3/1で既存の6に届かない。
したがってOtsu maskへ置き換えるだけの案には、この残差を改善する根拠がない。
anchor条件の数値をこれらの画像へ合わせて緩めたり、認識用の元RGBを二値化したりしない。

保存本文の再確認では、6列すべてで50/100のraw本文と既知CLI DEIM本文がbyte一致した。
FVC v2全件packetの実資料44 raw比較（6物理列/11採点ID×4 CLI経路）では、一葉3列の
bands両設定が改善し、地獄変3列およびDEIMは不変だった。既知scopeの診断であり、
新規6画像の精度改善へ計上しない。

[画素表・保存本文比較・Red/Green](benchmarks/accuracy-followup-review/README.md)を保存した。
