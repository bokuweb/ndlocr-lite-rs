# 閉じ括弧と単語の間の明示的空白を診断する

## OCR・実装前仕様

fc22d35の画線支持は本文/closerの高さと字数を確認するが、閉じ括弧の前後の空白の有無を直接確認しない。
モデルが印字された空白を省略した場合、文字距離が改善しても空白区切りの単語距離が悪化する可能性がある。
先行の `） 情報` 対照では成分数等で保持しないため、独立成分が明瞭なLatin本文でも確認する。
製品の方針はこの診断では変更しない。問題が確認されたら次の仕様とRedを先行して修正する。

最初のOCR前に5 closer `) / ） / 」 / 』 / 】` と、`closer+ABC / closer+空白+ABC /
ABC+closer / ABC+空白+closer` の20文字列を固定する。空白はASCII1個。
角ゴシックW3/明朝28px、白余白8px、PNG/JPEG75で80画像・160 member。
truthは描画文字列そのもので結果を見て変更しない。font/Pillow/generator/画像/truth/archive SHAを記録する。
通常fixtureテストで全組合せ・truth・全member・decode/寸法のRed→Greenを先行する。

比較方針は9f6a5e2の厳密支持baseline、現在fc22d35、画像fragment、一律text-only。
厳密支持の再構成は現在のbody fallbackを呼ばず、先に純粋Red→Greenと既存1271×3の実before本文との
全byte一致を検証する。現在方針も同じ原raw/RGBから前段の全実after本文へ一致することを要求する。
新80×30/50/100はraw候補APIを1 forward、現在の実Session APIを1 forwardで記録し、本文とconfidenceを照合する。
全step上位2確率・stop理由・原RGB/成分と候補を保存し、真値を方針選択には使わない。

既存1271＋新80=1351画像×3モデル×3正規化=12159の文字・単語指標を正式toolで全件採点する。
前段1271と新80、空白あり/なし、prefix/suffixを分けて退行と回復を記録し、同値変更を改善と呼ばない。
新240について旧厳密支持CLIと現在CLIも各240新規実行してAPI/再構成本文と照合する。
実行前にbinary/source/driverを凍結し、旧CLIのcompiled sourceと新APIのsourceを混同しない。
候補を比較する診断であり、この時点では現在版が新対照でも非悪化だと主張しない。
未知実資料holdoutや速度改善の検証とは数えない。全出力と失敗案を保存する。

空白なし40画像は前段literal corpusと同じ描画条件のpaired再現対照であり、独立の新資料とは数えない。
各対応IDをmanifestへ記録し、Rustでdecodeした画素と元rawの一致を検査する。
新しい明示的空白画像40と再現対照40は別に集計し、重複の改善を新しい改善として数えない。
