# 分離部品案の公開接続検証

適用するのは固定した `docs/detached_body_components.md` の支持条件だけ。
旧33,735組、最初の新32,832組、追加適用対照と元Horizontal指定の実NN比較がすべて非悪化で、
追加対照でも実際に適用される場合だけ、公開APIのRedから製品接続へ進む。
幾何検査で真値を仮rawにした結果は、実NNの精度や適用件数へ合算しない。

- 最初にInfo40の実Sessionでraw `情報)` が正読なのに処理後 `情報` となるRedを記録する。
  同じprivate支持を単行の最後に接続し、既存の包含/1画素案が成功した出力を維持する。
- 原RGB・前処理・モデル・デコーダ・confidenceとfragment用処理は変更しない。
  実NNで固定した全rawを製品へ再生し、診断案と全件一致させる。
- 全変更の実画像/該当モデルを公開Sessionで再認識し、原raw/本文/mean bitと一致させる。
  font/サイズ/spacing/palt/PNG-JPEG/mark形状/先頭末尾の境界を覆う負例も加える。
  データ全件の方針比較と、公開APIの実NN検査を区別する。
- 3モデルでSession/Pool/並列batch/単一Session batch/standalone、raw/候補/fragment、
  RGB不変、エラー後再使用、Cascade開始3/2/1を確認する。候補は常に3字なので25/45字の閾値は跨がない。
- 全変更の単行CLIと境界の代表負例を実行し、終端LFを1個だけ除いた本文を固定値と照合する。
- ページ経路はfragment用poolと単行用policyの呼び分けを確認し、通常/ONNX/例の回帰を実行する。
  必要なページの実行は、前段の287×8経路の全件新規実行や保存記録の再使用と区別して記録する。
  新しいページNNを走らせていない結果を、新規推論による検証とは表現しない。

新しい公開API経路や評価で退行が見つかれば接続を止め、反例と結果を保存する。
支持条件を同じ追加結果へ調整することはしない。

## 最終検証の記録

判定条件を変更せず製品接続まで完了した。全件結果、適用対照の限界、
実NN/API/CLIと再生の区別、再現手順は
[detached-body-components](benchmarks/detached-body-components/README.md) を参照。
凍結時点の先行仕様は同packet内の診断archiveにも保存する。
