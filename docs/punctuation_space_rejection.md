# 余白候補による自動空白削除を製品経路から外す

## 変更前仕様

Fは比例送りの11本文組・22文字/22単語指標で退行した。
元の非空白本文と全3候補のconfidenceを比較するGも、既存4,022入力で6本文組、
新576入力で26本文組を悪化させた。計4,598入力・41,382指標に各64退行があり、Gも棄却する。
平均confidenceと余白候補の本文合意は、意味のある空白の削除を支持する根拠として不十分だった。
候補・画像・truth・失敗例を維持し、閾値や文字列別の例外で回避しない。

- 製品のSession/Pool/batch/断片/standaloneからFの追加認識を外し、production sourceを9fc4528と同じ動作へ戻す。
  この段階で別の自動空白削除を導入しない。raw/候補APIと元のconfidenceを維持する。
- 句読点候補Fと成分高/幅/advanceはtest-onlyの診断として保持する。
  既存whole ROI exportの明示opt-inは保持し、既定の分割/認識は変更しない。
- 実際にF/Gが壊した30/50/100モデルの比例送り画像を、source ID・画像/RGB/font SHA・真値とともに固定する。
  正しいASCII空白を保つ公開SessionのRedを、各モデルで先行する。
- 回帰テストは同Sessionの再利用、ロード後モデル削除、raw/候補不変、不正入力後再利用、
  単行/全batch/Pool/断片、standaloneと3モデル混在cascadeの順序・confidenceを検査する。
  古いFの正例も別の入力として残し、rawにある誤空白を新しい推測で書き換えないことを確認する。
- 実CLIでも比例送り3反例と旧F正例・縮尺反例を前後比較する。
  この変更の精度主張は新しい正読回復ではなく、棄却した自動削除の退行をなくすこと。
- F/Gの全診断・source/binary/model/input SHAと再現用スコアを保存する。
  通常/ONNX/例・fmt/Clippyを検査し、9fcと同じproduction sourceの証明とNN測定を区別する。

余白だけによる空白削除の棄却後も、他の画線/検出/分割の残差に対する改善を継続する。

## 撤去結果

- 3モデルの公開APIと混在cascadeで4 Red・縮尺負例1 Greenを実モデルで確認してから撤去した。
  sourceと実libtest executableを実行前に固定し、撤去後は全5テストがGreenになった。
  単行・全batch・Pool・断片・standalone、モデル削除後の同Session再利用、候補APIとf32 confidenceを確認した。
- 実CLIはF/撤去後/9fc基準の3版×7入力、計21プロセスが正常終了した。
  3比例空白の退行を解消し、3つの旧F回復例の改善も撤回した。縮尺反例は不変。
  撤去後の全7本文は9fc基準とbyte同一で、末尾LFを厳密に1個だけ除いて比較した。
- `parseq.rs` は9fcとbyte同一、`cached.rs` も新しいtest-only module import以外はbyte同一。
  Fのselector/画像支持もcfg(test)へ限定した。製品は元の認識・画像支持に戻り、追加forwardはない。
- 通常639件、ONNXスイート556件、example23件が成功し、Clippyの新規警告は0。
  診断用envなしではno-opとなるテストも含むため、全件を実NN forward数とは数えない。
  実際の新規NN測定とRed/Green、CLIの件数は別に保存する。

この撤去は9fcに対する新しい精度改善ではない。Fで回復したページの誤空白も基準へ戻る。
以前から残る丸字体や挿入文字の問題を含め、別の根拠を持つ改善施策を継続する。
