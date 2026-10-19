# 末尾閉じ括弧の直前にある単語空白の回復

仕様は `docs/closer_word_spaces_probe.md` と `docs/trailing_closer_word_space.md`。
比較元はfc22d35。既存1271に、先に描画を固定したABCの80画像を追加したところ、
`ABC 』` → raw `ABC』` →製品 `ABC』` など、7本文組・14単語指標が旧厳密支持版より悪化した。
文字は改善しても単語が悪化するため、以前の非悪化という結果の対象外に新しい退行が存在した。
40画像は空白付きの新しい条件、40画像は以前のjoined対照とdecoded RGB/rawが同じで、独立標本とは数えない。

末尾1個の閉じ括弧と3字以上の本文に限り、独立成分の字数・位置・整列を確認する。
本文内の2個以上の小さい画線gapをreferenceにし、それより明確に大きい末尾gapだけをASCII空白1個として保持する。
細い罫線、極端に高い画線、遠すぎる画線では推測しない。不確かな場合はfc22d35の方針を保つ。
語句や閉じ括弧の種類の置換、追加モデル・forwardはない。共通の成分抽出の従来条件も維持する。

## 結果

候補出力を見る前に、DEF・20/40px・2font・5括弧種・PNG/JPEGと細/高い罫線の176対照も固定した。
既知fontの合成開発対照であり、未知実資料のholdoutではない。
合計1527画像×3モデル×3正規化=13743の文字指標と13743の単語指標が、fc22d35比で全て非悪化。
旧厳密支持版比でも全て非悪化となり、発見した14単語退行を解消した。

| モデル | 本文組 | 生文字/単語距離改善 | 完全一致への改善 | byte同一 |
| --- | ---: | ---: | ---: | ---: |
| 30 | 1527 | 27 | 20 | 1500 |
| 50 | 1527 | 10 | 10 | 1517 |
| 100 | 1527 | 19 | 8 | 1508 |

4581本文組は56距離改善・うち38完全一致、4525byte同一。既存1271の3813本文は全て同一。
176追加対照は33本文組改善、ABCの空白対照は23本文組改善。joinedの重複条件を独立改善に含めない。
raw正読を後処理で壊す組は今回全体でも330残り、元1271の289も残る。目的全体は継続する。

## 実行と検証

元1271の3813 rawは親証拠とSHA、全before/after本文・fragment照合で再利用。
ABC80はraw/processed各240 forward、DEF等176は各528 forwardを新規実行し、confidenceもbit照合した。
修正後の公開Sessionは全4581入力を新規実行し、候補本文と全byte、元rawのconfidenceと全bitが一致した。
全変更と176対照の和集合190画像×3モデルの570 CLIを前後とも新規実行し、APIと全本文一致。
CLIでは終端LFを厳密に1個だけ取り除き、元からある空白をstripしない。
最初のABC80にも旧厳密支持版/比較元CLI各240プロセスの独立照合を保存する。

通常全595件＋追加境界1件、ONNXのCI対象493件と明示rootでの全件再生1件、example18件が成功。
対象Clippyは本体既存18・lib test28警告（18重複）で新規警告0、fmt/diffも成功した。
環境変数なしでno-opの診断をONNX件数に含む。最初の全ignored実行は既存テストが要求するmodel/charset環境変数不足で停止し、
CIと同じ明示設定の対象を再実行した。

製品Redは `ABC』` → `ABC 』` の純粋画線と実モデル公開API。
閉じたgap/幅/高さ境界、語中/複数括弧/二重空白/RGB/方向、拡大・移動・明度PBT64例を検査する。
3モデルのstateless/Session/Pool/batch/単一Session batch、fragment、Cascade開始3/2/1、RGB/confidenceとエラー後再利用も検査。
本文22/23/24/42/43/44字の純粋画線では空白込みの字数を確認するが、今回の実画像で空白回復が
25/45字の再認識先を変えるケースの精度まで検証したとは主張しない。ページ全件の新規再測定・速度・未知資料の汎化も未確認。

実行前にコピーしたbinary/source/driver/model/input SHAを保存し、モデル/binary本体は配布しない。
`after-api-binary` とCLIはlint前の等価なlet-else版、`final-api-binary` は `?` に整えた最終製品sourceで、
全4581の原RGB/rawから選択とfragmentをモデル不要で再生一致した。最初の直接pathによるAPI pilotは中断し、
最終API比較はコピー済みbinaryで最初から再実行した。これらの記録を区別する。

全本文・truth・元画像/decoded PNG・候補確率/停止理由・API/CLI・正式9 CSV/9 MD・失敗/成功ログ・source/scriptを
`evidence.zip` に格納し、`evidence.json` がarchiveと全memberのSHAを持つ。
`capture_*` はfc22d35を比較元として動かした診断driverのsnapshotで、現在の製品へ直接適用する前提ではない。

```sh
python3 docs/benchmarks/trailing-closer-space/reproduce.py --output tmp/trailing-space-restored
```

このscriptは全SHAを検査して展開し、同梱の正式採点toolで全18表と監査をbyte再現する。
別rootで全memberのSHA、18表＋監査のbyte一致と、全4581の製品選択/fragment再生の一致を確認済み。
環境変数を明示した既存実資料テストも1件成功した。
Rustでdecoded PNGから再生する場合は、凍結した検証executableに
`NDLOCR_TRAILING_SPACE_VERIFY_ROOT` と `NDLOCR_TRAILING_SPACE_OFFLINE=1` を与えて
`verify_trailing_space_actual_api_and_frozen_replay` のignored testを実行する。

ONNX件数の訂正: archive内の `validation.json` は最後の5件が終わる前の488件という中間集計だった。
同梱の完了ログは493件成功を示すため本文を訂正し、`verification.json` にログSHAと完了集計を追記した。
archive・既存の本文/数値評価表・SHAは変更していない。
