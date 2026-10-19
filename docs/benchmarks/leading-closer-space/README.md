# 先頭閉じ括弧の後ろの単語空白を回復

仕様は `docs/leading_closer_word_space.md`。比較元ae42fce。
前段で残った新対照41組は先頭closerとASCII空白の組み合わせで、正しいraw `) ABC` から ` ABC` へ崩れていた。
前段1527画像に、OCR/候補評価前に固定したKLM・24/36px・2font・PNG/JPEGと左側罫線の176対照を追加した。
追加画像は既知fontの合成開発対照で、未知実資料holdoutではない。

## 先頭への単純な対称化を棄却

本文内2個以上の小gapより大きい先頭gapを空白とみなすだけでは、185本文変更・108文字指標/262単語指標の退行になった。
例は `）ABC` →raw `) ABC` →before ` ABC` →候補 `) ABC`。
全角括弧の字幅の右側の余白も大きなgapになり、語の空白と区別できない。
この方針はtest-onlyに留め、全本文・成分・正式18表と全退行を `failed-v1/` に保存する。

## 狭い先頭advanceの追加支持

第2候補は同じ成分数/x区間/本文整列とgapの条件に加え、closer画線の左端から本文画線の左端まで
（closerの画線幅＋gap）が本文最大高さ以内に収まることを要求する。
本文3字以上、本文の2個以上のgapは正で最大gapの4倍が字高以下、先頭gapは
max(2*本文最大gap+ceil(字高/8),ceil(字高/3))以上、closer幅>=ceil(字高/4)、closer高さ<=2*字高。
語句やcloser種類を置換せず、元closer＋ASCII空白1個＋元本文を保持する。
曖昧な場合はae42fceの方針へ戻す。末尾の空白・raw/候補・text-only/page fragmentとconfidence/RGBも維持する。
追加モデル/forwardはない。

この条件の出力を読む前にPQR・18/32px・2fontと左側罫線の176対照も追加固定した。
全1879画像×3モデル×3正規化=16911の文字指標・16911の単語指標がae42fce比で全て非悪化。
第2候補は評価後に真値や閾値で選び直していない。

| モデル | 本文組 | 生文字/単語距離改善 | 完全一致への改善 | byte同一 |
| --- | ---: | ---: | ---: | ---: |
| 30 | 1879 | 26 | 25 | 1853 |
| 50 | 1879 | 26 | 26 | 1853 |
| 100 | 1879 | 26 | 26 | 1853 |

5637本文組は78距離改善・うち77完全一致、5559byte同一。65はraw正読の後処理損失を回復し、
12はモデルが省いた空白を補って完全一致、1は `)POR` → `) POR` で空白だけ改善し、誤字O/Qは維持した。
変更はABC12・DEF24・KLM18・PQR24の各本文組。元1271画像の3813本文は全て同一。
最初の41組中35組を回復し、残る6組は全角括弧側の空白で、この回復を完了扱いにはしない。
新対照を含むraw正読の未回復は全体400→335で、元1271の289も残る。目的全体は継続する。

## 実行と検証

元1527のraw4581は親archiveのSHAと全before/fragment/実API結果照合で再利用。
KLM/PQR各176はraw/processed各528 forward、合計1056ずつを新規実行した。
製品適用後は全5637を公開Sessionで新規実行し、全本文が候補とbyte一致、全confidenceがrawとbit一致した。
全変更と新PQR176対照の和集合194画像×3モデルの582 CLIを前後とも新規実行し、全APIと一致した。
CLIの終端LFを厳密に1個だけ取り除き、元の空白をstripしない。

純粋画線と実モデル公開APIの `) ABC` → ` ABC` のRedを先行しGreenにした。
gap/幅/高さ/advanceの閉じた境界、全角字幅の反例・語中/二重/非ASCII空白・RGB/方向・PBT64例を検査する。
3モデルのstateless/Session/Pool/batch/単一Session batch/fragment・raw/候補・エラー後再利用・Cascade開始3/2/1も確認。
通常603件、ONNX CI対象503件と明示rootのモデル不要再生1件、example18件が成功した。
環境変数なしでno-opとなる診断もCI対象件数に含む。fmt/diff/対象Clippy成功、新規警告0（既存本体18、lib test28/18重複）。
比較元ae42fceの全5CIも成功した。

本文22/23/24/42/43/44字の純粋画線では空白を含む字数を確認したが、今回の復元で実画像が
25/45字をまたいで再認識先を変える場合の精度は未検証。全ページの新規再測定・速度・未知資料汎化も主張しない。

## 証拠と再現

`evidence.zip` に元画像/decoded PNG・真値・全方針の本文を含むraw/成分/確率/停止理由JSON・実API・全CLI・
36評価表・監査・失敗/成功ログ・source/binary/input/model SHAを格納する。
方針別TXTはJSONから同梱scriptで生成し、重複archiveを省く。モデル/binary本体は配布しない。
元forwardと製品検証binaryは実行前にコピーしてSHAを固定した。同じ製品binaryで全5637の選択/fragmentも再生一致した。

```sh
python3 docs/benchmarks/leading-closer-space/reproduce.py --output tmp/leading-space-restored
```

全archive/member SHAを確認して展開し、同梱の正式採点toolで成功案・失敗案の36表と両監査をbyte再現する。
Rustでのモデル不要再生は `NDLOCR_COMPACT_LEADING_SPACE_VERIFY_ROOT` と `NDLOCR_COMPACT_LEADING_SPACE_OFFLINE=1` を設定し、
凍結executableの `verify_compact_leading_space_actual_api_and_frozen_replay` を明示実行する。
診断のbefore captureはae42fce sourceを前提としており、現在の製品へ直接適用する前提ではない。
