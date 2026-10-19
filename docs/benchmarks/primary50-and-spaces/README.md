# primary50ページ監査と生空白の余白合意案

仕様: [`page_primary50_audit.md`](../../page_primary50_audit.md) / [`raw_space_margin_consensus_probe.md`](../../raw_space_margin_consensus_probe.md)。
製品の変更はない。50文字モデルの未評価範囲を埋め、残る生空白の補正候補2案を棄却した。
既知実資料・固定フォント/変形を含む開発回帰であり、未知実資料holdoutの証拠ではない。

## 50文字モデルの公開API比較

1a66fceと4ba9fd6のproduction sourceをGitから別ディレクトリへ取り出し、
同一の新しい `capture_page_matrix` exampleを加えて、targetも別にしてビルドした。
実行前に実executableをコピーし、source/driver/Cargo/model/charset/全入力SHAを固定した。

既存287画像をprimary50、parallelism=1、thread=1で、評価設定
（DEIM .25/min line .3/drop empty true）と既定設定（.3/0/true）で両版とも新規実行。
全574本文組・全行metadataがbyte/値一致し、none/width/strictの
**1,722画像別文字指標・単語指標は全て非悪化**。精度改善とは数えない。

新driverの30/100による5画像×2設定×2モデル×2版=40対照も、
前段の保存済み実APIの本文・bbox・confidenceと完全一致した。
旧100モデルの対象画像の誤読と、現100モデルの回復も両方再現している。
既存出力先への再実行は拒否し、12既存ファイルのSHAが変わらないことも検査した。

前段6経路の5,166比較を、元archive/入力とproduction sourceのSHA一致を確認して再利用すると、
8経路で6,888文字/単語比較が非悪化となる。新規測定は50の2経路と40対照。
前段6経路を今回も新規推論したとは扱わない。

## 空白補正案は両方とも棄却

元の生本文が内部のASCII空白1個を持つ6〜12文字の和文だけを対象に、
左右2px・上下0/1/2pxの3候補すべてが同じ非空白本文を返し、confidence .98以上で、
上下0px候補が元のconfidenceを厳密に上回る場合、その実モデル候補を選ぶ案Aを固定した。
案Bは、原RGBの内部完全白列群の最大幅が0超・画像高の3/4以下であることも要求する。

次の3段階で条件を変更せず比較した。各入力の4RGB条件をatlasへ画素そのままで並べ、
明示bboxを1区間ずつ3モデルへ渡している。全1,208区間は各1chunkだった。
3段階計302入力のうち新規は300、既知の対象2区間を含む。各区間の元画素・追加純白と、
atlasの全切出しRGBをSHA/画素で検査し、計3,624区間認識の全候補/停止理由/確率を保存した。

| 対照 | 入力 | Aの生距離改善 / 退行 | Bの生距離改善 / 退行 | Bの完全一致回復 |
| --- | ---: | ---: | ---: | ---: |
| 読点後の空白0/1/2個、3字体、24/36px、原寸/半分、PNG/JPEG75＋既知2区間 | 146 | 48 / 42 | 48 / 0 | 48 |
| 読点後以外の全文字間の実空白、3字体、紙余白0/2px | 66 | 33 / 0 | 33 / 0 | 0 |
| 読点後の実空白への紙余白2/4/8pxと、実際の語間空白 | 90 | 0 / 19 | 0 / 16 | 0 |

第2段の33改善は空白を誤位置に出した元出力の距離2→1で、実空白の欠落は残る。
全段階を合わせたAの退行は122文字/122単語指標、Bも32文字/32単語指標。
いずれもnone/widthで現れ、ASCII空白を除くstrictでは隠れる。合計改善で退行を相殺しない。
最初のBの48回復だけを採用根拠にせず、両案を製品へ追加しない。

具体例は `予定時刻、 担当者`。上下各2pxの白紙を追加した原入力は正読（約.98695）でも、
余白候補は `予定時刻、担当者` と誤読し、confidenceは約.99967へ上がる。
前景高と内部白列幅はどちらも12のまま、画像高だけ14→18になりBのgateがfalse→trueとなった。
紙余白に依存しない画線高の支持は別の仕様/対照で検証する。

## 証拠と再現

`evidence.zip` は対応する `evidence.json` に全archive/member SHAを持つ。
ページの原画像/真値/全本文・bbox/confidence、実行前source/binary/model SHA、
Gitから取り出した両版source、新driver/全コマンドとRed/Greenログ、40対照の参照値、
3段階の原画像/atlas/全候補・確率/選択/公式表/棄却例を含む。元fixtureの出典READMEも同梱する。
model・実binary・フォント本体は配布しない。

```sh
python3 docs/benchmarks/primary50-and-spaces/reproduce.py tmp/primary50-and-spaces-restored
```

新規ディレクトリへ全memberをハッシュ検証して展開し、標準ライブラリだけで
50モデルの12表と空白診断54表、計66CSV/MD・4監査を再計算してbyte一致を要求する。
40対照も参照値と比較し、前段との6,888比較の集約範囲を検査する。
別ディレクトリで全5,522 member・66表・4監査と40対照の一致を確認した。
RGB検査の再実行にはPillow、実モデル再実行には固定assetsとONNXビルドが必要。

新exampleのmanifest検証2件、ONNX全example20件、fmt/diff/Clippy成功。
新規Clippy警告0（既存lib18）。4ba9fd6のCI5ジョブも成功した。
単行/ページの製品sourceを変えた新たな改善、未知実資料への汎化、速度の改善は主張しない。
