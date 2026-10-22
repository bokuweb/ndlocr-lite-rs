# Root接続の完了gateを機械的に検証する

実装前仕様。GitHub jobの成功だけではRootへ接続しない。compareは悪化をsummaryへ記録して
process自体はexit0となり得るため、実集計のcomplete/full_gate_passを要求する。
v2の26,220画像結果/79,020 CSV行、悪化0、Page本文/bbox/f32 bits変化0を整数で完全一致させる。
旧v1の24,780/74,700、partial、欠けた字段、false pass、悪化やPage変化をすべて拒否する。

この純粋guardを未実装Red→Greenにし、Root適用script/実行driverから呼ぶ。
加えて実run39593424953/head fe0e359・全59 jobs成功、独立全56 worker replayを別に確認する。
このguard自体はモデルを呼ばず、完了件数や悪化を補完・推定しない。
