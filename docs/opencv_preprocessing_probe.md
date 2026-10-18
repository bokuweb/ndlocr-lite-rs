# 本流 OpenCV 前処理の比較診断

## 実装前の仕様

本流 `ndl-lab/ndlocr-lite` の `d001dad2013ceabb5736ff8d637474e443eb73c2/src/parseq.py`
は uint8 RGB を `cv2.INTER_LINEAR` でリサイズした後、BGR/f32/NCHW へ変換する。
Rust の浮動小数点補間との差が認識に影響するかを、製品設定を変えずに調べる。

診断専用 example は OpenCV 4.12.0 (`opencv-python-headless==4.12.0.88`)、NumPy 2.2.6 の
独立プロセスを oracle として使う。Rust で OpenCV の丸めを近似しない。
Python 依存を製品や公開 API に追加しない。横書きの同一 RGB・背景処理・分割・crop・モデル・
文字集合・decoder を既存の比較器と共用する。方向の自動回転は比較対象に含めない。

入力は little endian u32 の幅・高さ・出力幅・出力高さ、続いて正確な RGB bytes。
応答は little endian u64 の byte 長、続いて BGR/NCHW の little endian f32。
Rust は寸法・RGB 長・overflow を書込み前に検証し、応答長不一致、途中 EOF、非有限値、
正規化範囲外を拒否する。worker の異常終了を成功として扱わず、終了時に回収する。
これらの失敗テストを先に追加する。独立した OpenCV 画素 fixture で色順・配置・丸めを照合する。

モデルを読む前に omo 側で既知14画像・既存合成36行・記号3帯の全条件と採点器を凍結する。
既存 bilinear の全文・信頼度・座標・分割の再現、各画像の誤り非増加、完全正読対照と AI 句の
保持、既知および合成の誤り総数の減少をすべて満たした場合だけ次の製品実装候補とする。
既知14画像と既存合成画像は探索済みであり、未知実文書 holdout の代用にはしない。

同一 CPU 1 thread/Level3 session で実行順を交互にする。全結果と失敗条件を保存し、上書きを拒否する。
Python IPC を含む前処理時間は製品速度の予測には使わない。認識に効果が確認できた場合に限り、
独立 fixture に一致する pure Rust 実装を次の Red とする。

参照: https://github.com/ndl-lab/ndlocr-lite/blob/d001dad2013ceabb5736ff8d637474e443eb73c2/src/parseq.py
