# Latin pair案の公開API接続仕様

追加rawの判定中も製品へは接続しない。既知の元RGB fixture
klm_round_suffix_mincho24.pngで、固定モデル3種のrawがKLM)、
既定APIがKLMになることを先にRedとして記録する。
元meanのfloat32 bitとraw文字列・RGBを固定し、既存の公開API診断loggerで照合する。
これは新規raw評価60,228 forwardとは別のAPI検証呼出しである。

新規対照全件非悪化と未重複RGBの実NN適用が通過した場合にだけ、
単行の最後のfallbackへ固定済みbool支持を接続する。
同じseedと旧新の全変更組・事前固定した図形対照のAPI/CLIをGreenにする。
新規対照に退行があれば接続せず、Redの証跡と失敗例を保存する。

## 永続する実モデル回帰

同じfixtureを用いるignored testをtests/support/latin_pair_model.rsへ追加する。
3モデルのraw Auto、既定Session Auto、Pool Horizontalを各1回実行し、
全raw・処理後文字列を集めてから既定本文の保持をassertする。
RGB不変とmeanのfloat32 bitも照合する。既存CIの--lib --include-ignoredで実行する。
接続前の実行では全3モデルのraw正読と既定本文の欠落を同時に記録する。
