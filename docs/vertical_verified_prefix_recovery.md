# 元raw全文と二分割合意を保つ縦列容量回復

## 実装・新画像・追加NN前の仕様

先行prefix候補Jは字形切断でNN前に棄却した。Kも幾何gate未達、Lは切断0を達成したが
認識18入力model組の退行で不採用だった。Lの分割を既存の公開splitterへ一律適用しない。
今回は安全な分割を新しい回復候補の内部だけで使い、元raw全文を一字も変えない合意付き延長を
通常Fragment+Autoへ接続できるか検査する。Hのcount<=16/group40/4倍/0.9/空白列条件は変えない。

### 対象と出力metadata

- 同じ借用sessionの元raw結果が成功した後だけ検査する。元認識の失敗は元のErrを保つ。
- 選択されたoutputの固定T/Cとcharset長が一致することを、既存の静的上限helperで調べる。
  model名やinput幅からTを推定しない。動的/不明/不正shapeは候補なしとする。
  T>=3、元rawのUnicode scalar数がT-1またはTの場合だけ候補を作る。
- 元rawがUnicode空白で終わる場合は候補なし。既存Fragment処理が末尾で除去する空白を、
  延長で内側へ移して新たな語境界にしないためである。元raw自体をtrimして比較しない。
- 原幅>=12、Autoと同じ縦回転範囲、元の長さがモデル入力比の2倍を厳密に超す場合だけ扱う。
  RGB/寸法/積の不正、0のmodel幅/高さは候補なし。span予算を整数で半分にするため、
  Jの試作にあった奇数model寸法の拒否は使わず、奇数寸法も幾何テストへ含める。
- 通常SingleLineと明示Horizontalは従来のまま。Hが採用した結果や既存sanitizeの位置も保つ。
  呼出しは借用済みsessionのraw Horizontalへ渡し、追加loadや再帰lockを行わない。

### 新しい候補内の支持と分割

元RGBを既存の無補間CCW回転へ渡す。支持用だけ既存の有理数Otsuを使い、NNには元RGBを戻す。
支持インク列は全幅1/3以上、字高支持は全高1/2以上を必要とする。
Lのanchor/pitch/薄い独立画線の保護を、候補内だけのprivate helperとして使う。
既存公開splitter/Hに使われる旧分類は維持する。画素走査の共有が必要なら、まず原走査の
byte同一の抽出と純粋テスト/既知幾何replayで既存挙動を確認する。

主分割の最大spanは既存と同じ`min(20*列高, 2*列高*model入力幅/model入力高)`とする。
副分割は、この主span予算を半分にする。単にmodel入力幅を半分にする方法は、20倍の上限に
両方が当たるmodel100で同じ分割になるため使わない。20群制限/分割探索/空白を足さない規則を保つ。
両分割は2chunk以上で、異なる境界集合であることを必須とする。

### 固定する合意と延長の条件

- すべてのchunkが非空白、finiteのconfidence0.9..1、raw scalar数1以上T未満。
- 両分割を元順に連結した全文がbyte完全一致し、元raw全文を完全一致のprefixとして含む。
  少なくとも1 scalar長く、異なるalphanumericが4種類以上あることを必要とする。
- Unicode空白scalar列が元rawと完全一致することも必要とする。trim/空白削除/句読点の推測や
  字種置換はしない。語彙・GT・font/PID/行番号を条件に渡さない。
- confidenceは主分割だけのscalar加重f64平均をf32へ戻す。副分割を独立標本として数えない。
- 失敗/不一致/不支持は元rawをそのまま保ち、部分出力を返さない。
  合意しても元rawの文字を直す候補は不採用にする。

### Test firstと未生成対照

Jのprefix/空白/失敗/原RGB保持テスト、Lのanchor/pitchの12テストとPBTを再利用する。
別sourceコピー内で、private支持と旧公開分割を分離する小さいRed→Greenを先行する。
full/half spanが3model形状で異なること、unknown T/no-op、同じsession内の回復失敗/元Err保持を検査する。

走査抽出後のdefault検査では、既存の空白候補FのPBTがproptest内部のfloat sampler assertionで停止した。
OCR関数のassertionではない。固定乱数streamによる再現試験では同じassertionは再現しなかった。
同じ有限区間としきい値隣接値を整数bit samplingで生成し、property本体・製品処理・依存versionは変えない。
追加した無entropy stream試験は整数rejection samplingで停止しなかったため、その自分のPIDだけを終了した。
この失敗source/log/PIDを保存し、明示的な境界と隣接f32値の8×8直積試験へ修正した。
修正後は元64case propertyを含む7試験が通過した。

Lの全198RGBを既知回帰として残す。新しい3段落を30/31/35、50/51/55、100/101/105 scalarへ
切り出し、既知3書体×余白2条件の54本文を作る。T-1に相当する完全な本文も負側の境界対照として
必ず含め、延長不要な完全文へ文字を追加しないことを検査する。段落には内側のU+3000空白も含める。
3背景×2図形×31/51/101群の新18図形も固定し、診断上のGTは空とする。
新72RGB/SHA/GT/glyph bbox/fontと生成条件をNN前に固定し、新旧の重複を拒否する。
既知fontの合成開発対照であり、独立資料holdoutや学習未使用とは主張しない。

全270RGB・注釈162本文について、主/副spanと元RGB/Otsu支持の幾何を先に検査する。
字形内部の切断が1件でも残れば候補全体をNN前に棄却する。GTで入力ごとの採否を選ばない。
実際の候補到達数は元rawを得てから記録し、dummy文字で開いた幾何plan数と混同しない。

### NN・再利用・採否

元raw Autoは全270×3modelを揃える。旧54×3の保存rawは、原RGB/寸法/方向と同じ
Session/preprocess/decoder/model/charsetであることを確認できる場合だけ再利用してよい。
過去Lのraw Horizontal chunkも、同じmodel/RGB/寸法/方向の場合だけ再利用し、共有を新NNと数えない。
新しい呼出し集合・採用条件は追加NN前に固定し、GTをmodel入力へ渡さない。

全270RGB/modelの候補有無、全chunkのliteral本文/平均bits/停止/失敗、追加API呼出しを保存する。
モデルload/画像API/内部forward数は区別し、未計測のforward数を推定しない。
全入力別none/width/strict文字・語の退行0、負例の出力安定、完全文への誤延長0、少なくとも1新規RGBの
改善を採用条件とする。既知/新規を合わせた平均だけを採用理由にしない。

条件を満たした場合だけ実モデルnative Red→GreenでFragment+Autoへ接続する。
raw Session/通常SingleLine/明示Horizontalは既存値を保持し、Fragmentの単件/batch/専有sessionの
出力/平均bits一致を全270RGB/3modelで確認する。Hの既存採用/棄却も既存sourceと対応させる。
その後、既存293ページ・既知縦書き7ページ・新対照の実API/CLI A/Bと必要スイートを行う。
退行や無改善なら接続せず、J/K/Lと今回の全反例・source・raw・失敗を保存する。
