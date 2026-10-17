# PARSeqの文字候補と画像由来の確率

文脈によるOCR補正の採否には、原画像から得た代替文字の根拠が必要。平均信頼度だけでは文字ごとの候補を比較できないため、既存の後処理前デコーダと再利用セッションへopt-inの候補APIを加える。今回、認識本文の自動補正や言語モデル依存は追加しない。

`predict_text_from_flat_logits_with_candidates` は既存本文・平均信頼度と、各時刻の上位k候補を返す。候補はtoken ID・文字（EOSのみNone）・全クラスsoftmax確率を持つ。上位候補内で確率を再正規化しない。順序はlogit降順、logit同率ならtoken IDの昇順。kは1以上、クラス数を超す値はクラス数へ切り詰める。反復文字・空白・全角半角を変更しない。

各時刻は出力本文内のUTF-8 byte rangeを持ち、EOSまたは低信頼度で停止した時刻にも候補を返す（本文範囲はNone）。停止理由はEOS・既存の低信頼度条件・全時刻消費を区別する。EOS以降や低信頼度停止以降は返さない。既存の「2文字以上出力後に最大確率0.35未満なら停止」を維持する。候補はPARSeqの同一forward出力の周辺確率であり、代替本文を再推論した系列確率やOCR向けに校正済みの正答確率ではない。

新APIは非有限logit、形状不正、文字集合とクラス数の不一致、k=0を明示的に拒否する。宣言形状内を検証し、旧API同様に形状外の末尾データは無視する。旧APIの入力許容範囲・本文・計算順序を変えず、候補配列の確保・選択・ソートは新APIだけで行う。

`ParseqSession::recognize_rgb_u8_candidates_with_orientation` は既存の方向指定・前処理・ロード済みモデル・排他制御を共用し、1画像につき1回だけ推論する。候補取得のための再ロード・再推論は不要。失敗後も同じセッションを利用でき、AutoとHorizontalを呼び出しごとに選べる。

Redから、全クラスsoftmax・同率・Unicode byte範囲・反復・EOS/低信頼度・不正入力を固定する。PBTで旧本文/信頼度との一致、候補数・順位・確率・定数シフトの不変条件を確認する。固定モデルで反復数字と横書き数字の既存結果、ファイル削除後の再利用、失敗後の復帰を確認する。既存全文回帰で本文・座標の維持を確認する。次段階は、凍結した実OCRと正しい原文の負例で、文脈候補の回復数と誤補正数を測る。

## 利用例とコスト

```rust,ignore
use ndlocr_lite_rs::infer::{cached::ParseqSession, parseq::ParseqOrientation};
let session = ParseqSession::load(model_path, charset_path)?;
let evidence = session.recognize_rgb_u8_candidates_with_orientation(
    rgb, width, height, ParseqOrientation::Horizontal, 3,
)?;
for step in &evidence.steps {
    println!("{:?}: {:?}", step.text_range, step.candidates);
}
```

候補要求時は全logitを検証し、クラス数Cのインデックス領域を再利用する。処理した時刻数Tに対し、保存量はO(C + T × min(k,C))。各時刻は上位kの選択とその部分だけのソートを行い、全クラスの候補オブジェクトは保存しない。順序はlogit降順、logit同率ならID昇順（softmax値が浮動小数点の丸めやunderflowで同じ値になっても、元logitの順序を保つ）。通常APIでは候補領域の確保・検証・ソートを実行しない。候補収集の追加時間は未測定であり、無料とは扱わない。
