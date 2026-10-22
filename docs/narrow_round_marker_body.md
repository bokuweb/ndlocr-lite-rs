# 狭いround profileと接触本文のtest-only結合

## 本文結合実装前仕様

比較元は1ec730a。`narrow_round_marker_profile.md` の数値を保ち、mark単体の13テスト/PBT64通過後に本文側と結合する。
新しい入口はtest-onlyとし、最後の単行fallbackを含む製品経路には接続しない。

- `owned_parts_body_bridge` の既存入口はstrict H/4幅のままとする。
  既存のcomponent viewをstrict/narrow profileの別入口から共有し、本文側の処理を複製・変更しない。
- 新入口は空白なし3 scalar、端の `)` / `）` が正確に1個、本文2 scalarは表示幅2。
  内部/複数closer・他closer・Latinは候補にしない。元fragmentを保持するだけで文字を追加・置換しない。
- filtered成分3〜6、markと本文のgap0/1、既存のmark最大幅/高さ/非矩形判定を維持する。
  新入口だけはH/4最小幅の代わりに凍結したnarrow round profileを全て必要とする。
- 本文2〜5成分のunion、唯一の主成分、全FG所属、同じ1画素だけのcut、中心距離、左右group、
  大きな3辺内包/1辺共有部品、既存supports_collectedの重なり判定は変更しない。
- 既知 `)情報` で新入口がfalseのRedから進め、既存strict入口はfalse、製品本文は従来の `情報` のままであることも確認する。
  共有化後は既存/新profile全テストと、保存137,991組の実製品再生が1ec730a固定値と一致することを確認する。
- private比較では凍結した1ec730aと新案を全保存rawで比較する。元Auto非回転/明示Horizontalだけを対象にし、
  fragmentの文字数が既存製品より多く新入口の画像支持がある場合だけ保持する。
  none/width/strictの文字・単語指標に悪化があれば本案を停止し、同じNN結果へ条件を合わせ直さない。
- 全体非悪化の場合だけ、新NN前に追加対照と元RGB重複・未重複集計を別途規定する。
  未重複RGBへの実NN適用と全群非悪化が確認されるまで製品へ接続しない。
