# FVC v2のRoot接続とローカル回帰

全件run39593424953の59 jobs成功、79,020 CSV行/39,510比較の独立検算、悪化0、Page変更0を
確認してからguardを通して接続した。Appとfinishは凍結v2のまま、既存Root32の等価iteratorと
test-only geometry登録を保つ。src全92ファイルの橋渡しを検証した。

| 完了したRoot検査 | 結果 |
| --- | --- |
| CLI既知literal（DEIM off/on） | 先行2 Red → 新CLIで2 Green |
| 既存finishとの品質処理一致 | 3設定/3 RGB、本文とf32 bits一致 |
| quality強制設定 | initialからbitsが変わり、finish省略を検出できる |
| ONNX通常lib | 468成功 / 114 ignored |
| 通常lib | 456成功 / 39 ignored |
| CLI parser integration | 10成功、埋込planの384 argvを含む |
| ONNX無効のCLI | 16 process / 8比較、exit/stdout/stderr/body完全一致、NN0 |
| fmt / default・ONNX all-target Clippy | 成功、新FVCファイルの警告0（既存警告は保持） |

Root v3のlib/CLI/testはown artifactすべてfresh=false、852 file/resourceを前後固定した。
CLIは46,638,576 bytes / SHA256 `769dce6873b69bc1104f7a264e210b39308c9c8262dfd9610d0b9a5e9558632f`。
libtestは81,963,712 bytes / SHA256 `4eb5c1778b1d7bfecfe57baa73c1d5bd5685af84ec8e2d6ce3edb5b2bfa3a288`。
回帰test binaryはbeforeと同じ429a743b…で、実CLIだけを取り替えたRed→Greenである。
literalは120 scalarでGT121 scalarの既知空白欠落は残す。新規改善へ数えない。

初回quality回帰は旧Python配置のcharset既定値でNN前に失敗した。
test-onlyで固定3モデル/charsetを絶対パス指定するよう修正した。
続くpacket readerは旧847 fileのfreezeが、新quality test/fixture4 fileを欠くことも検出した。
旧v1/v2 freezeとテスト成功は履歴に残し、完全なsource固定の成功へ数えない。
source列挙helperを2 missing-module Red→2 Greenで作り、literal include先のoption ZIPも含む
852 fileをRoot v3で凍結した。fresh v3の3実行物はv2とbyte一致し、runtimeは変わっていない。
新ScopeでCLI2/quality1/lib/feature無効16を再確認した。

採用対象のpacketは `root-bridge-v1/completed-local-v2.zip`（594,869 bytes、SHA256
`9a2c3ed2ed0e05c3149c5a1b3d816672bc1a92c9fe20837193db1d8278f3a827`）。
旧 `completed-local-v1` はsource不完全のstatusを明記した。
readerは2 missing-module Red、初回provenance assertion failureを保持し、最後の2 tests Green。
全文/framing/品質3設定/crop座標/f32 bits/全source/fresh compiler/16条件をNN0で再検証する。
不足・不一致・非有限bits・finish無効果も拒否する。

必須CIにはLinux/macの実CLI2回帰を独立stepとして追加し、stdout/stderr/本文/argvを保存する。
known qualityもignored libを含む既存実モデルstepで必須実行する。全5 jobs成功後に次候補Rの
runtimeへ進む。それまではRの72対照・geometryのNN0準備だけを維持する。
