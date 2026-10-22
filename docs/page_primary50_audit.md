# 短い語尾の再確認を50文字モデルで監査する

## 実装・推論前の仕様

- 比較は短い語尾の再確認を含まない `1a66fce` と、採用済みの `4ba9fd6`。
  既知287画像・固定した真値・DEIM/charset/modelを引き継ぎ、再描画しない。
- primary50のPageRecognizerをparallelism=1で使い、評価設定
  （DEIM .25/min line .3/drop empty true）と既定設定（.3/0/true）を実行する。
  同じ供給モデル以外を追加ロードしない。通常の製品処理は変更しない。
- 公開APIで任意の固定manifestを記録できる診断exampleを追加する。
  推論前に空manifest・重複ID・出力先を逸脱するIDを拒否する純粋テストを先行する。
  入力はid/imageだけを使用し、truthや既存の推論結果を認識に渡さない。
  各設定の本文（行をLFで連結）・行数・全bbox/confidenceを保存する。
- 比較元は必要なproduction sourceをGitの指定commitから別ディレクトリへ取り出し、
  同一の新example/driverだけを追加してビルドする。元checkoutを切り替えない。
  ビルドのtargetも版ごとに分け、別sourceの成果物を共有しない。
  推論前に実executableをコピーし、source/driver/Cargo/model/charset/全入力SHAを固定する。
- 新driverの30/100による対照を、前段で取得した3対象画像とmanifest先頭/末尾画像で
  両版・両設定とも実行し、既存の実API本文・bbox・confidenceとの一致を要求する。
- primary50は287画像×2設定を両版とも新規実行する。公式scorerのnone/width/strictで
  全1,722文字指標・単語指標を比較し、合計による退行相殺を認めない。
  変更がなければ精度改善とは数えず、未評価だったモデルの非退行確認として記録する。
- 既存6経路は前段archiveのSHAを確認し、測定時sourceを区別して再利用できる。
  8経路の表を作る場合、50以外も今回新規推論したとは表現しない。
- 実空白を削る語句固有の置換は追加しない。退行・新しい誤読が見つかった場合は
  原RGB/crop/生モデル/結合処理を切り分け、変更前に別の仕様と失敗テストを固定する。
- 全て既知資料・合成/変形を含む開発回帰。未知実資料への一般化・速度は別評価とする。

## 結果

40対照の本文/bbox/confidenceは前段と完全一致。primary50も全574本文組・
全metadataが変更前後で一致し、1,722文字/単語指標は全て非悪化だった。
精度改善とは数えない。前段6経路の5,166比較と合わせた8経路では6,888比較が非悪化。
前段のproduction sourceと今回の両版sourceのSHAも一致を検査した。

新しい診断exampleのmanifest検証2件、ONNX全example20件、fmt/diff/Clippyが成功。
製品のソース変更はなく、4ba9fd6のCI5ジョブも全て成功した。
証跡は `docs/benchmarks/primary50-and-spaces/` に保存する。
