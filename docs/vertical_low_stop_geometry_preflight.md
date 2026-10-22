# S0の1/4 spanを隔離helperへ追加する（実装前仕様）

最初のモデル不要実行は、既存private helperがspan divisor1/2だけを受け付けるため、4でexit101になった。
これは字形切断や認識の悪化ではなく、未対応パラメータによるprototype構築の失敗である。
partial ZIP/source/binary SHA/終了値を保持し、追加NNは0のまま。

隔離したgeometry-sourceの受理集合だけを1/2/4へ広げる。元のmax budgetを算出してからdivisorで
割る既存算術と、全境界探索/画像/模型寸法は変更しない。0/3/5/usize::MAX等は引き続き拒否する。
本体のhelperやH/Oは変更しない。

先にuniform glyph blocksのquarter分割をRedにし、64例のsize/group/平行移動propertyで
全chunk境界がインク内部に入らないことを検査する。実装は受理集合の一行変更にとどめる。
その後、固定した既知1組のdivisor2/4を新しい出力先で記録する。
divisor2はR0の既存境界/寸法/実RGBと全件一致し、divisor4も独立PIL回転/cropと一致が必要。
glyph内部切断が一件でもあれば追加NNへ進めない。

元のS0仕様の合意/EOS/文字数/confidence/元Fragment保持の条件は変更しない。
新しいquarter partをGTに合わせて選び直さず、全unique RGBの実認識・失敗・停止を保存する。
