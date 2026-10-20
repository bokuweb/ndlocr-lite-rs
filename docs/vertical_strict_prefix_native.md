# O: 固定した完全prefix延長だけをnativeへ接続する

## 実装前の仕様

N2は414RGB×3モデルで入力別none/width/strict文字・語の退行0だったが、新72RGBの改善0だった。
Nの一文字訂正4組は依然として提案時に既知の入力だけであり、N2の事前採用条件を満たさない。
N全体と訂正分岐は採用保留とし、しきい値を緩めない。

OはMで固定した完全prefix延長の方針を変更せずnativeへ接続する候補である。
M作成後に別本文・配置で生成したNの72RGBで、完全prefix延長が7組改善した。
これはMの旧270RGBでの改善2組と分けて報告する。N2の新72RGBでは改善0であることも残す。
既知fontの合成開発対照であり、独立実資料holdoutではない。

### 固定する処理

- `vertical_verified_prefix_recovery.md`のMのplans/combine、支持、主/副span、原RGBをそのまま使う。
- 元raw全文をbyte完全一致のprefixとして保つ。既存文字の訂正は、確信度によらず拒否する。
- 選択outputの静的T/Cとcharset、実logitsのTを照合する。T>=3、元scalar数T-1/Tだけ。
- 全chunkはraw HorizontalのRecognizeResultを同じ借用sessionから得る。候補traceの再decodeは要らない。
- Hが採用した結果を優先し、sanitizeの位置、SingleLine/明示Horizontal/raw Sessionを保つ。
- 元Err/不明metadata/不正画像/途中chunk失敗は元挙動を保つ。部分回復は返さない。
- Mからのprivate splitterの走査抽出とlegacy公開挙動を保つ。語彙・GT・モデル名で採否を分けない。

### Test first / 採否

1. Mのactual Rust combineを全414RGB/3modelの保存rawで再生し、旧270/N72/N2 72の結果を分ける。
   方針のsourceがMと同じことをhash/最小diffで検査する。保存traceからrecognitionだけを使う。
2. nativeの公開APIで、新N完全prefix成功をRedとして先に確認する。訂正が必要なN4組は元の出力を期待する負例とする。
   metadata不明、callback失敗、境界、Unicode空白の既存純粋テスト/PBTを維持する。
3. public PNG/期待文字列を同梱したignored実モデル回帰を追加する。既存CIのonnx lib --include-ignoredで実行する。
   新N31/35文字の完全prefix成功と、既知一文字訂正例の拒否を含める。モデル30の既存環境変数を使う。
   他OSでの浮動小数点bitsを固定しない。raw/通常/Fragment本文、finite confidence、失敗後再利用を検査する。
4. 全414×3×8 API経路をe5製品と比較する。Nのruntimeとは異なるため候補matrixを流用しない。
   baseline旧342はe5source/RGB同一の場合だけ再利用し、新72は測定する。
   H保存、Fragment単件/batch/専有batch一致、通常/Horizontal/raw5経路不変、非文字114入力不変、完全文54対照不変を必要とする。
5. 入力別none/width/strict文字・語の退行0と、M固定後の新N改善7組の保持を必要とする。
   既知訂正4組は不採用のまま。新N2改善0を隠さない。293ページ/既知縦7ページ/新API/CLIと必要suite、再現packet/CIまで通してから本体採用する。

画像APIと内部forward/load数は分け、未測定forwardはnullとする。既存の失敗/反例/原データを保持する。
