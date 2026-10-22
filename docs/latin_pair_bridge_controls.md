# 英字pair支持案の追加NN前対照

## 生成・NN前の固定条件

比較元はd711eec。既存162,915保存raw組の製品本文維持と候補の全群非悪化を確認してから生成する。
固定したpolicy/kernelの数値を変更せず、全19,440条件を保持する。
既知の字形・fontによる開発対照であり、未知資料のholdoutとは扱わない。

- 描画modeはFreeType BASIC/palt未指定、CoreText/palt0、CoreText/palt1。
  前段と同じ3font file/face0、24/40px。実font/face/glyph/advance・元native PNG/traceを保存する。
  FTでnumeric glyph IDやpaltを捏造しない。全文字がmissing glyphでないこととfallbackなしを検査する。
- 本文はKLM/XYZ/MLK/WM/VVM/qjp。WMは2文字、VVMはWとの過分割・反復、qjpは字高の対照である。
  文字条件は本文内ASCII空白0/1（全隣接文字間）、round closer2種、前後、端空白0/1、
  上下紙0/2/10、PNG/JPEG75(subsampling2)の全組10,368入力。
- bare本文の本文内空白0では全6本文、空白1ではWMだけに、9種の外側図形を置く。
  none/1px線/bar/4x4点/斜線/chevron/left_curve/fork_inner/fork_outer、
  前後・gap0/1・PNG/JPEG75の9,072入力。真値は元本文だけの診断上の規約である。
  forkは記号にも見えるため意味が一意な実資料の非文字GTとは扱わず、全例を保持する。
- 実native描画は文字1,728+bare126=1,854回（FT618、CT1,236）。別の1,854再描画で検査する。
  全非白画素のbboxを2px白紙で囲うROIを原画素のまま作り、内部advance/実空白を保つ。
- 図形の基準高Hはbare ROIのRGB和660未満の前景bbox高とする。
  bar/斜線/chevron/curve/forkは幅max(3,ceil(H/4))・高さceil(4H/3)で、Latin markerの寸法域も調べる。
  thin幅1、dotだけ4x4、noneは0x0。Hが6未満なら生成を停止する。
- 斜線はfloor(y*(W-1)/(MH-1))、chevronは両左端から中央の1頂点へ進む丸めた1px線。
  right curveは両側floor(MH/3)のflankでxを線形に動かし中央を平らにする。
  left_curveはその左右反転。fork_innerはright curveの中央行にmax(0,W-4)、
  fork_outerはfloor((W-1)/4)の枝を追加し、直前行の枝と右端の間を1px線で接続する。
  内側/外側の両forkを保持し、結果で容易な方だけを選ばない。
- 図形は元本文の全非白bboxの外側に置く。縦位置は前景bboxの中央へ合わせ、
  必要な白紙を上下に加えて元本文と図形を全て保持する。4,536 pre-JPEG組を独立再構成し、重なり0を検査する。
- 全tuple/元RGB/有効RGB/真値/atlas/描画記録を照合し、108例以上を目視する。
  全162,915親組と(width,height,RGB SHA)で照合し全一致元を保存する。同じRGBの異なる真値は停止して記録する。
  条件/model/方向/重複を独立標本と数えない。全mode・本文・文字/図形・空白・重複/未重複を別集計する。
- NN前に真値を仮rawとする幾何到達性を確認する。未重複RGBで成立0ならNNを実行しない。
  これは認識精度ではない。全元Auto/有効Horizontal tensorを3形状でbit照合してから全ROIを認識する。
  Auto回転する原入力は全て元Horizontal指定でも測定し、独立画像と数えない。
- 追加rawは19,440x3=58,320 forwardと元Horizontal指定分。旧新全群・none/width/strictで
  入力別の文字/単語退行が1件でもあれば本案を製品へ接続しない。全元入力/失敗/反例を保存する。
  未重複RGBへの実NN適用と全群非悪化まで精度改善や製品採用を主張しない。新NNの結果へ条件を合わせ直さない。

## 公開API/CLI検証の採用前固定条件

製品へ接続する場合は、旧新の候補変更組を全件、Auto/元Horizontalの該当方向で
raw API・既定APIに通し、raw文字列と候補文字列・平均信頼度を凍結値に照合する。
加えて新図形条件はmode/font/size/body/body-space/shape/lead/gap/formatの
各組につきmodelをidのSHA256先頭byte mod3で一意に割り当てた1モデルを全件検査する。
CLIは全変更Auto組と、この図形対照の同じ1モデルを全件検査する。
結果を見てサンプルを選び直さない。API呼出しはraw収集forwardとは別の検証呼出しとして数える。
