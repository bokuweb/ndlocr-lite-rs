# 固定kernelの既知成分への読み取り専用適用

## 計測前仕様

kernelの13テスト/PBT64例とClippyを通過した条件をkernel-lock.jsonで凍結する。
27元RGBの前段projectionを入力に、test-onlyの実Rust collectorとkernelを呼ぶ。
NNを行わず、文字列を救済する候補や精度測定とは扱わない。

- 元RGBの寸法・SHAを実行前後で検査する。実行source・仕様・入力と実行ファイルのSHAを先に保存する。
- collectorのbbox/面積/成分数を前段の無除去projectionと照合する。この既知群は全成分が十分大きい。
- 全84成分を記録し、各bbox内の全前景数がcollectorの面積と異なれば所属不一致としてkernelを適用しない。
  外接矩形の重なりで別成分の画素が入る場合も記録し、行を捨てない。
- kernelの基準高には各成分自身のbbox高を使う。これは成分内の分割可能性だけの監査であり、
  将来の本文全体の基準高やmarkerとの対応が決まったことにはしない。collector全体の高も別に記録する。
- 元bbox・面積・所属判定・基準高・成立/不成立・成立したlocal/global viewを保存する。
  元mask/RGBは変更せず、成立時はview面積の和+1が入力前景数に一致することを検査する。
- 既知のLMとXYについて、固定kernelが予期したbboxの2 viewを返すことを先にテストする。
  この結果で条件を新NNへ合わせ直したり、文字境界が一意に分かったとは主張しない。
