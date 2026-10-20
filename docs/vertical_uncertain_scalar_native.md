# 候補Nの実Fragment経路への接続

## 本体接続前の仕様と段階

`vertical_uncertain_scalar_recovery.md` の固定案は全342画像/3モデル/3正規化で
文字・語の退行0、非文字変更0、完全文36対照の誤変更0だった。
実Rustの採用13組のうち7組はN新規RGBで、各正規化でも改善した。
実際のFragment+Autoへはまだ接続していない。これから別sourceコピーでnative Redから始める。
既知fontの開発対照という範囲は維持する。

### 元認識のtrace

- `ParseqPool` のFragment+Autoだけを対象とする。同じ借用sessionで一度得た元logitsを使う。
  元の `predict_text_from_flat_logits_with_confidence` とそのErrを保ち、同じlogitsのtop-1 traceを必要時だけ追加decodeする。
  trace取得のために元画像を再推論せず、追加load/再帰lockをしない。
- 選択されたoutput名のshapeとcharset長を既存の静的上限helperで調べる。
  別outputの固定shapeへfallbackしない。不明/動的/不正shape、C不一致はN候補なし。
  実logitsのTも同じ静的Tであることを確認する。model名や入力幅からTを推定しない。
- T>=3かつ元raw scalar数T-1/Tだけtraceを作る。traceの本文/mean bitsが元rawと異なる場合はNを使わない。
  元rawのErrは通常どおり返す。追加trace decodeだけの失敗は元rawへfallbackする。
- 通常SingleLine/raw Session/明示Horizontalの推論とdecoderは従来の経路を保つ。

### 回復と後処理の順序

- 既存Hを先に評価し、採用された結果はそのまま使う。Hの条件/source/meanを変えない。
- Hが採用しなかった場合だけNのplanとtrace条件を検査する。
  新しいprivate分割だけを使い、旧公開splitterにはLの一律変更を入れない。
- 主/副chunkは元RGBの無補間回転/cropとし、同じ借用sessionのraw Horizontal/top-1 decoderへ渡す。
  chunk API/decoderの失敗は元rawへ戻し、部分出力は返さない。
- Nの合意/一文字の確率/厳密延長/空白列/weighted confidenceは凍結した案と同じとする。
  既存Fragment sanitizerはH/Nの後の同じ位置で適用する。

### 小さいRed → Green

1. output選択の純粋テストを先行する。選択名だけを参照し、動的/不正/不在で候補を作らない。
2. 画像回復のcallbackテストを先行する。元RGBの座標/両分割順序/失敗時fallbackと、不正trace/未知TのNN前棄却を確認する。
3. nativeのFragment採用ケースを、固定NN結果のliteral本文/mean bitsで先に失敗させる。
   新規対照の正例を含め、model30/50で実際に到達したケースを使う。
   model100にはN採用例がなかったため、存在しない正例を作らない。
4. cached hookを最小変更で接続してGreenにし、元raw/通常SingleLine/Horizontal/エラー再利用とHを比較する。

### 接続後に必要な検査

全342×3モデルについてFragment単件/batch/専有sessionの本文とmean bitsが固定方針と一致すること、
他経路が変更前の実出力と一致することを保存する。Fragmentの元基準はHを含め、rawだけの比較と混同しない。
元H採用/棄却と、293ページ・既知縦書き7ページ・新対照の実API/CLI比較も行う。
CLI単行の比較では追加された終端LFを1個だけ除き、rawやGTをstripしない。
必要な通常/ONNX/example/fmt/ClippyとPRのCIを確認し、凍結source/全成功・失敗・raw/確率/検査を再現可能に保存する。
これらが終わるまで製品の採用・改善完了とは呼ばない。
