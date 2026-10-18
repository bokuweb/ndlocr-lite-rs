# 左右補間色外縁の局所検証

仕様は `../../lateral-panel-fringe.md`。最初に例示/PBTを失敗させた (`red.log`, `red-tests.rs.txt`)。最小実装後2件Green、負例を含め19件Green (`green.log`)。外側の孤立色文字による追跡中断も追加Red (`red-isolated-ink.log`) 後に、横64画素以上の背景一致区間を起点にして8件Green (`green-isolated-ink.log`)。

初回の一定色4画素帯の負例は旧8f641bdでも失敗する既存挙動だった。`negative-tests-initial.rs.txt`, `negative-fixture-initial.log`, `baseline-negative-fixture.log` に残し、行ごとに混合率を変えて既存の全高単色枠の背景延長と区別した。

黒/異色罫線、厚い/短い縁、囲まれた白字形、紙RGB、249成分の紙を検査。32ケースの色/平行移動PBTを含む。通常cargo testの初回全体実行は成功 (`cargo-test.log`)。起点選択の最終変更後は左右8件と既存上下・外周紙・近白紙を合わせた20件が成功 (`green-final.log`)。全体はCIで再検証する。ClippyはCI相当で成功、既存6警告が残る (`clippy.log`)。

実画像 `ocr_formatted_accuracy_+2` の凍結済み斜行分割では修正前後とも14行で、16行の目標は未達。境界と文字の重なり、背景標本の不一致が残るため、条件を広げない。固定28画像の全文・認識時間・座標・負例の比較は omo #79 で行う。独立holdout、JPEG一般化、全体精度、production readyは未確認。
