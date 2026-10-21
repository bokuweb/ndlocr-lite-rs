# ndlocr Python→Rust移植（ONNX）計画

## 2026-10-21: RootのCLI回帰2 Redとknown候補2 Greenを固定

- 現在のFVB Rootからlib/CLI/testをfresh=falseでbuildし、844 source/fixtureを固定した。新CLI2テストは実CLIのexit0/本文生成後にliteral不一致で2 Red。凍結v2 CLIへ同じtest binaryを渡すknown precheckは2 Greenで、Root runtimeは未接続。
- 既知arial121はGT121字、固定raw期待値120字（U+3000欠落）だった。会話中の字数表記を訂正し、GT/期待値の内容・SHAを維持する検証を追加した。初回readerの字数assert失敗も保持し、最終7 packet tests Green。新72/全件の採点は元GTで変更なし。詳細は`docs/vertical_cli_known_literal_length_audit.md`。
- feature無効Root基準もfresh lib/bin、NN0で固定した。known qualityの固定PNG/sourceを準備し、全件成功後に登録して必須モデルCIへ組み込む。接続仕様は`docs/vertical_cli_fallback_v2_root_bridge.md`、現状は`docs/vertical_cli_fallback_v2_root_preparation.md`。69bcc35のCIは全5 jobs成功。

## 2026-10-21: 実primary分割の117 raw結果を検証し、raw保持条件の緩和を見送る

- 元36条件に対応する18入力から2経路/117 partを生成し、全117 API成功、4,224,384画素の独立座標一致、297公式CSV行の独立検算を完了した。model30の両経路全文一致は3列だが、完全raw prefixを保持する延長は0列。一葉の途中欠落を読めても空白位置/字体が変わるため、合意を作る正解依存の補正は行わない。
- Rootはtest-only geometry collectorだけを追加。未実装compile Red→2 pure Green、通常lib442成功/39ignored。全source/fresh build/入力/117 decoder step/297 CSVを公開readerで検証する。新規精度改善0で、H/O/Q/Sの製品条件は不変。詳細は`docs/real_vertical_primary_partition_results.md`。
- ec9d276の通常/実モデルCIは全5 jobs成功。FVC v2の全件39593424953は継続中で14/56 worker成功、失敗0。本体はFVB。全件成功後に最小runtime差分を接続し、必要CIを通す。

## 2026-10-21: 追加実資料の実停止理由を全36条件で照合

- 6列×3モデル×2設定を18 model/RGBへdedupしてraw候補を再観測し、元Pageの全36本文/f32 mean bitsと一致した。EOS17、LowConfidence1。一葉30のcol1/2はEOS、col3だけ低信頼停止だった。停止理由を最終欠落から推測せず、実stepで確認した。
- 補助処理3 Red→3 Green、packet reader未実装Red→4 Green。全RGB、36対応、全before binding、source/fresh build、f32逐次meanを公開packetから検証可能。これは原因診断で精度改善0。次は別分割の安定性を診断し、S/Q/H/Oの条件を既知GTで調整しない。詳細は`docs/real_vertical_additional_stop_probe_results.md`。

## 2026-10-21: FVC v2設定gate完了、拡張全件gateを開始

- 非default384 CLI/1,152 CSV行/576比較は全件完了し、入力別文字/語悪化0、reject_allの64 processは空本文。公開ZIPの全argv/本文/採点コピー/GT/整数指標/比較一覧をreader未実装Red→6 packet tests Greenで独立再検証した。新規精度改善へは数えない。詳細は`docs/vertical_cli_fallback_v2_option_results.md`。
- v1全件run39568828241は成功し、74,700 CSV行・悪化0・Page本文/矩形/信頼度bits変化0。798 raw採点経路gainは既知scopeの値で新規798画像ではない。v1は既存finishを省略するため採用しない。
- v2の1,311物理入力/1,317採点ID、26,220結果/79,020 CSV行はrun39593424953（fe0e359）で開始。初回39591981411はbuild checkout中、推論開始前に中止した。共通runner末尾の旧74,700固定assertをplan由来の件数完全一致へ変更し、2 Red→2 Green・旧/新契約計19件Green・推論処理不変のsource bridgeを保存した。本体はFVBのまま、全件成功とroot接続検証を待つ。

## 2026-10-21: FVC v2新72が成功、実CLI parserを設定gateへ追加

- 再測定した新72は576 CLI結果/1,728 CSV行を完了し、独立距離と全誤り/分母が一致した。入力別文字/語悪化0、10本文画像のraw22経路組改善（bands各10、DEIM各1）。非本文36枚は全経路で不変。旧v1の14画像/30組は二重計上しない。本体はFVB。
- 1.3MBの新72 packetへ全完了結果と停止129 receiptを併せて保存し、reader未実装Red→4 packet tests Greenで864比較/全argv/body framingも再計算した。停止分は採用数0。詳細は`docs/vertical_cli_fallback_v2_results.md`。
- 設定384の最初のprocessはquality boostの値付きargvをCLIが拒否してexit2、画像認識/model load0だった。元計画/失敗を保持し、実Rust Cli::try_parse_fromへ全384を渡す回帰をRed→10 CLI tests Greenにした。quality trueだけ値なしflag、falseは既存defaultとして省略するv2計画へ修正。
- 凍結前後CLIの48設定vectorは、存在しない画像でparser後の画像読込error/NN0を確認した。初回wrapperのerror文言期待失敗も保持し、再検算readerをRed→5 packet tests Greenにした。Python設定契約も5 Green。推論/GT/モデルを変えず、新計画/新出力先で384実画像processを実行中。

## 2026-10-21: FVC採用経路の既存後段を保持するv2

- source監査でv1の採用経路がrecognize_initial_batchの後を再利用し、既存quality boost等のfinishを通らないことを確認した。default全件run39568828241はv1の凍結記録として継続し、それだけではv2や設定保持の成功へ数えない。本体はFVB。
- `docs/vertical_cli_fallback_postprocessing.md`を先行し、finish呼出し/エラー保持の2 assertion Red→10 Green。既存loopをbyte同値（rustfmt後は空白を除き同値）で共通化し、initial batchはbyte不変。通常lib453成功/40ignored。compile fixture不足と2回のENOSPCも保存し、同じsourceを再試行した。
- v2新72を追加NN前に固定し、4,392 glyphの独立描画と既知1,239入力とのPNG/RGB重複0を確認。公開packetは22,479,201 bytes。最初のarchive member絶対pathをreaderが拒否したため、入力/GT/source byteを変えずmember名だけを正規化し、元archive/hashも保持した。reader未実装Red→Greenで全member/fresh build/PNG/RGB/字形bboxを認証した。
- 既知quality回帰はcharset不足1回と、設定効果を判別できない入力2回の失敗も保持した。既知12字列の3設定では既存full pipelineと全本文/mean bitsが一致し、delta=-100では初期から信頼度が変わりfinish省略を検出できた。DEIM off/onのliteral CLI回帰2 processも成功。9保存設定/9 RGBをreader Red→Greenで再検算し、新規精度改善へ数えない。
- v2新72の前後576 CLI process/1,728 CSV行は別に測定中。全入力別文字/語非悪化と新本文改善が揃うまで次の全件gateへ進めない。root e5c3173の通常/実モデルCI全5 jobs、追加実資料のモデル不要reader CIも成功した。
- 最初のv2新72 runは容量不足が起きた間にPython exit120で停止。129成功baseline receipt/stderr空を保存し、元例外は復元不能として0完了gate扱いにした。閉じた自分のRGBをSHA/復元台帳付きで可逆圧縮し、全元bindingの一致と1GiB以上の空きを確認して、出力先だけ変え全576を再測定している。partial予測は流用しない。
- 非defaultは8入力×6profile×4経路×前後=384 argv/1,152 CSV行を設定NN前に固定した。plannerの網羅/未知入力拒否2 Red→2 Green、明示引数1 Red→3 Green、公開plan reader1 Red→4 Green。新72 gate成功前は設定認識を開始しない。
- feature無効は新v2 default CLIのown lib/bin fresh=falseを確認し、元FVB default CLIをSHA完全一致で復元して16条件のexit/stdout/stderr/body不変を確認した。NN/model load0。readerは未実装Red→3 packet tests Green。これはONNX精度gateとは別である。

## 2026-10-21: 追加実資料の途中欠落をnative traceで切り分け

- 追加6列を既存P5/O観測実行物で新たに36 Page測定し、FVB本体baselineの全本文/行数/bbox/f32 confidence bitsと一致した。全primaryはDEIM検出で、FVBとの差があるbands fallbackを通っていない。実PARSeq forward66試行/66成功、PageRecognizer load6/model session load12を区別する。
- primary36/O part30の独立RGBと、全174捕捉RGB、before/traceの396 CSV行を検算した。一葉model30はraw23/22/11 scalarでH非採用・O容量条件外。Q/Sの既存適格条件にも届かず、容量上限より短い途中欠落への別案が必要。decoder/Hの具体的停止理由は未観測で断定しない。
- 地獄変model30のO分割は空白差/不合意/raw文字変更で棄却された。GTを見た文字置換は行わない。全source/失敗/結果を内容address packetに保存し、未実装reader Red1→Green1、関連モデル不要11試験が成功した。詳細は`docs/real_vertical_additional_primary_trace_results.md`。本体FVBとC1/Q/S保留は不変で、FVC全件run39568828241は継続中。

## 2026-10-21: FVC新72が先行gate成功、C1新96は改善0で保留

- FVC新72×CLI4経路の前後576 process/画像結果を完了。公式/独立1,728 CSV行・864採点比較が一致し、文字/語の入力別悪化0、14本文画像のraw30経路組が改善した（bands on/off各14、DEIM各1）。全文/argv/sourceとbefore-NN packetの対応をreaderでも再検算した。本体はFVBのまま。
- 最初の全件run39565834405はJPEGにも.png名を付けたharness不備でdecode失敗となり中止。46 worker partial/build/logを保存し、完了worker0として全件結果へ数えなかった。混在formatの2 Red→11 Greenで修正し、全1,121 PNG/118 JPEGの元byte/SHA/signatureを確認。推論/モデル/GTを変えず新runで再測定する。
- FVC全件gateを1,239物理ID/1,245採点IDへ拡張固定した。C1新96は入力だけ追加し、C1方針は混ぜない。新72の改善を再度新規へ数えず、24,780画像結果/74,700公式CSV行とPage本文/矩形/confidence bits不変を必要とする。9 model-free契約テスト、56完全shard、認識前argv計画、partialを保持するretry選択を準備し、Linux dispatchへ進む。仕様は`docs/vertical_cli_fallback_full_gate.md`。
- C1既知CLIは24→71字のliteral Red1→Green1だったが、新96前後768結果/2,304 CSV行は全384 raw本文組が不変。悪化0でも新本文改善0なので全C1を保留し、全既知比較や本体接続へ進めない。source/入力/失敗/全結果を保持した。詳細は`docs/vertical_cli_model_agreement_results.md`。
- root 6fcbe76の通常/実モデルCIは全5 jobs成功。最新の本体推論はFVBと等価iterator整理のままで、FVC/C1の隔離sourceを本体へ入れた結果ではない。改善作業は継続する。

## 2026-10-21: C1をNN前に固定、FVCと別の単列モデル合意案

- `docs/vertical_cli_model_agreement_proposal.md`を先行し、initial100の支持不足だけを30/50全文一致・元全文prefix/空白保持・画像anchor/高confidenceで再確認するC1を隔離実装した。本体FVBや進行中FVCとは合成しない。callback1 Red→Green、棄却3 Red→Green、model shape/元crop2 Red→8 Green（128例PBTを含む）を確認。
- 新96（本文48/反復12/横書き12/非文字24）をOCR前に固定し、PNG/RGBと4,860 glyphを独立照合した。既知1,143物理IDのheaderも別に固定し、hash重複0。初回描画の縦横比assertion失敗・source・部分画像は保持してから、横書き負例のrow advance64を明示した。OCR結果を見た調整ではない。
- 隔離CLI接続後のdefault448成功/38ignored、ONNX lib Clippy既存18警告、両variantのown lib/CLI/test各fresh=falseを確認。全default初回のfixture不足32失敗も保持し、root f99の561fixtureをSHA照合して解消した。初期cascade・既存後段とFVB helper本体はbyte不変（helper/moduleの可視性だけ変更）。
- 約16.6MBのbefore-NN packetとreaderを公開した。readerは未実装Red→96画像/全member/全字形/非重複/fresh buildをGreenにした。C1のモデル認識はpacket作成時0。実CLI既知例のRed/Greenと新96の全件gateは次であり、精度改善や採用とは数えない。

## 2026-10-21: CLI traceが完了、initial100の早い停止を確認

- Linux run39552644890で既知3画像×3設定×2 sourceの18 CLI processを完了。通常/観測の本文・stdout・stderrがすべて一致し、27 crop RGBと54 CSV行を独立照合した。観測した非空pool batch9と、未計測の内部forward/sessionを区別する。
- default3画像はinitial100だけを使い、promotionはなかった。30/50が同じ71字を返したbold例で100は24字だったが、他の2例では30が失敗し、50にも誤りが残った。新規改善の根拠にせず、一律のモデル切替は行わない。詳細は`docs/vertical_cli_cascade_trace_results.md`。
- 初回は既存include_bytes!用PNGの同梱漏れでcompile停止/CLI0だった。元packet・失敗artifactを保持し、同じ基準commitのfixture追加をRed→Greenにした。推論Rust/モデル/3入力は不変。モデル不要readerも未実装Red→全件Greenになった。
- refactorの最終通常lib440成功/38ignored、ONNX all-targets Clippy成功（lib18/lib-test30、17重複込み）を保存。1547b32のroot CIも全5 jobs成功。FVC新72比較は別runで継続し、その後の全件条件を`docs/vertical_cli_fallback_full_gate.md`に先行記述した。

## 2026-10-21: FVB本体の実モデルCI成功、追加実資料baselineとFVC実測

- e559efbの全5 CI jobsが成功。Linux/macOSのONNX lib各563件と、必須モデルを使うPage回帰各1件も通過した。FVBの全921 NN入力/922採点ID×8経路gateは全44,256 CSV行・22,128前後比較を独立検算し、入力別悪化0/新本文raw7組改善/CLI不変で本体へ接続済み。
- 追加実資料は事前固定した6物理列/11採点表記をe559のfresh実行物で測定し、48結果/264 CSV行が独立距離と一致した。1 baselineの観測であり新たな改善比較ではない。Page30の途中脱落、50/100/CLI DEIMの空白・字体・少数誤字、CLI bandsの構造的失敗が残る。生出力を保持し、NFCだけの別診断も公開する。詳細は`docs/real_vertical_additional_results.md`。
- FVB新ファイルのONNX Clippy警告1件は、先に128例PBTを旧実装で通してから等価iteratorへ置換。既存13件を含む14件を再確認し、lib警告は既存18件へ戻った。推論の改善件数へ数えない。
- 隔離FVCは既知CLI回帰で旧版のliteral本文Red1→候補Green1を確認し、候補はDEIM有無の本文/stdout framingを通過した。新72×CLI4経路の前後576 processを実測中で、完了・入力別悪化0・新本文改善が揃うまで採用しない。本体はFVBを維持し、Q/Sは保留、CLI model traceも別準備として継続する。


## 2026-10-21: FVB Page30の全件完了と、時間上限時の完全分割の準備

- 本体推論はO/1f240beのまま。91e09ebの通常/実モデルCI全5 jobsも成功。91e09ebでQ/S保留のsource/入力/失敗/全結果を公開した。新しいNNを使わないUbuntu CI/run39529309504が成功し、全20,736公式CSV行・50,653捕捉crop RGB SHA・全packet/memberを独立検算した。
- FVB原run39514388558のPage30が完了した。921物理/922採点ID・3,684 Page結果についてsource/fresh artifact/argvと全本文を確認し、公式11,064 CSV行の独立誤り/分母が一致した。生文字改善35入力/経路組、悪化0。50モデルの全件jobも完了したが独立採点はこれからで、100/CLIは実行中。全44,256行のgateは未成立。
- Page30の前後認識が約2時間20分かかったため、未完了kindが180分を超えた場合に限る分割再実行を`docs/vertical_page_sharded_matrix.md`へ先行固定した。同じ921入力順の4分割で231/230/230/230件を前後とも新測定し、完了したkindの元binding付き結果と結合する。画像/経路/GT別表記や入力別非悪化条件を減らさない。
- 分割/網羅4試験、producer/失敗保持2試験、NN前planの存在とSHA結び付き、欠落拒否と未分割同値をRed/Greenした。既存14契約を合わせ23試験が成功。4入力/GT5表記の13ジョブを結合し、全240 CSV行が未分割と一致、欠落shard/planを拒否した。成功load数も実行形態に合わせて合計する。
- 元のNN producer/build/CLI/画素や本文reader主要関数はsource完全一致で、source/argv検証と採点だけを共有化した。共通化後も完了Page30の実artifact全3,684結果とbindingを再検査した。まだ追加NNを起動せず、原runの未完了kindを確定してから必要分だけ再測定する。


## 2026-10-21: FVB全件CI、Q保留と低信頼停止Sの隔離検証

- 本体推論はO/1f240beから不変。PR22の06401d7は通常/実モデルの全5 CI jobs成功。FVBはfbd9896の凍結packetから921 NN入力/922採点ID×8経路をLinux/releaseで再測定中（run39514388558）。同じpacketの重複run39514635185は取り消し、未完了結果を成功へ数えない。
- FVBの公開Page回帰は別Linux CI/run39514635451でOの本文assertion Red1件/FVB Green1件を実行した。source/fixture/test/bundle/fresh artifact対応を再検算し、raw build/test記録を別SHA付きarchiveへ保存した。全件gateはまだ必要。
- P5は羅生門3列×6設定の18 Page結果を観測し、全本文/矩形/mean bitsが既存Oと一致した。18 primary/12 O part RGBは独立PILと一致、実PARSeq forward30試行/30成功。1列・model30の2設定だけO planが成立し、両分割が全文で合意したがraw途中欠落と空白差のため棄却した。他16は容量付近条件に不適格。個別glyph欠損は未注釈null。
- `docs/vertical_unique_internal_insertion.md`のQを実装/新対照/追加NN前に固定した。O優先、既存plan/NN部分結果だけを再利用し、唯一の内部連続挿入で元rawの全字/空白を保つ。純粋helperは5 Red→9 Green、支持/meanは3 Red→5 Greenで計14試験（各512例の2 property）を通した。NNなしの既知部分結果105組のRust/独立採否・mean bitsは一致し、採用は既知羅生門1列/2設定のみ。これはcounterfactualであり実Page結果とは区別する。
- 隔離Qを実Pageへ接続し、同じ既知例でO Red1件/Q Green1件を確認した。元ASCII空白・矩形・confidence bits、eval/default、不正入力後再利用を保持し、生文字距離6→1/語2→2。通常lib426成功/38ignored、ONNX nonignored438成功/112ignored。本体へは未接続。
- 新Q72（本文54/図形18）をNN前に固定し、3,348 glyphの独立配置が一致した。旧198+新72の6,480幾何記録/26,506 cropを独立検算し、supported glyph切断0/legacy156を区別、旧198の4,752記録は公開canonicalと一致した。新72×3model×8 nativeの前後比較を完了し、3,456画像結果・10,368公式CSV行を独立検算した。全1,728前後組で本文/mean bits不変、新本文改善0のため全Qを保留した。既知1列だけの改善で採用しない。
- R0は既知H残留誤り3組をGT選択した診断で、22 raw Horizontal cropを実測した。二分割合意で安全な挿入は成立せず、新しい方針を採用しなかった。S0はそのうちLowConfidence停止1組だけを細分化し、追加8認識・既存4再利用で両routeが同じ50 scalarを返した。元H43 scalarを残す7文字内部挿入が唯一になったが、既知例の診断として扱う。
- Sの先行仕様`docs/vertical_low_stop_recovery.md`はH実採用・静的/実shape一致・実LowConfidenceを必要とし、divisor2/4のEOS・高信頼・非飽和・全一致から一意内部挿入だけを許す。隔離コピーで停止metadata wrapper、純粋採否、元RGB再認識呼出しを小さくRed/Greenし、24試験を通した。旧1/2 geometryの共有化前後も486 RGB×3 modelの1,458組で準備RGB/境界を完全照合した。追加NN前に新72を固定した。全558 RGB・注釈378枚・1,674条件の候補1,141条件で24,147 cropがPILと一致し、glyph内部切断0。実native既知例はRed1→Green1で7文字を回復、ONNX非ignored462成功/115 ignored。新72×3model×8経路の6コマンドを完了し、3,456画像結果と10,368公式CSV行を独立照合した。全1,728前後組が本文/mean bits不変、新規改善0で全Sを保留した。本体へ未接続。詳細は`docs/vertical_low_stop_results.md`。
- 別OSのbuild記録のfilesystem resolveを避けるreader回帰と、PR全体のpathsに加えて今回pushの差分を見るCI guardをRed/Greenで追加した。モデル不要14試験が成功。改善全体は継続中。

## 2026-10-21: FVAを非文字への幻字と本文悪化で保留、FVBを別検証

- FVAは全240入力×Page API6経路で47入力/経路組の本文を変えた。公式/独立CSV8,640行は一致したが、10入力・19経路組（3正規化で57件）で悪化したため全案を保留する。悪化を伴わない28組の改善で相殺しない。詳細は`docs/vertical_page_fallback_results.md`。
- 非文字に高信頼の「AAAA」「NNNN」「the the ...」が生じ、本文1組も文字誤り97→98へ悪化した。幾何の字形欠損0やconfidenceだけでは方向fallbackの採用根拠にならなかった。本体推論は1f240beのまま。
- `docs/vertical_page_supported_output_proposal.md`のFVBを別案として固定した。画像anchor数に対する非空白scalar支持と4種類のUnicode alphanumeric支持を追加し、3 Red→13 Green、refactor後13 Green、通常lib439件・ONNX nonignored451件・実Page既存回帰1件を通した。ignored件数は別記する。
- 新規108対照（多様本文60/繰返し12/縦長横書き12/図形24）を認識前に固定。全918 ID/917 unique RGB×raw/preparedの1,836ケースで920候補を選択し、独立crop/anchor数・全非白画素・glyph検査が一致、字形欠損0・横書き誤昇格0だった。これは幾何だけの成功。
- FVBの348入力×6 Page API先行比較が完了した。新規baseline108とcandidate348の計2,736 API結果を測り、O baseline既知240の1,440結果を再利用した。公式/独立12,528 CSV行が一致し、全3正規化の文字/語で入力単位の悪化0、新規本文5画像・raw7経路組が改善した。既知font合成対照での開発結果として扱う。
- ローカル全件比較は容量枯渇時に中断し、324件の部分本文を成功全件に数えず保存した。`docs/vertical_page_remote_matrix.md`を先に固定し、全921 NN入力/922採点ID/920 unique RGB×8経路をLinux/releaseの4独立CI jobsで前後とも再測定する。Macの予測を流用しない。NDL『羅生門』1頁の3列も含み、曖昧字のGT二表記は両方で非悪化を必要とする。
- `docs/benchmarks/vertical-page-fallback/`に凍結入力/source/開発記録/幾何を保存した。モデル不要11試験と既存採点5試験が成功した。全44,256公式CSV行の独立照合、入力単位の回帰0、新規本文改善、CLI本文不変を必要とする。全件gate、実Page新規回帰のRed/Green、必要suite/CIが揃うまで本体推論へ接続しない。改善全体は継続する。


## 2026-10-21: Oのfresh Page比較が成立、方向fallbackを次候補へ

- 実行物取り違えの旧「全件不変」は撤回したまま保存した。freshな73e9ab4の全444 ID/443 unique RGB×8経路を再測定し、公式/独立21,312文字・語比較で退行0を確認した。model30のeval/defaultで各9入力が改善し、各設定・各正規化の文字誤り13/語誤り9減。50/100とCLI両経路は不変。行数/矩形は不変、平均bitsは改善した18組で変化した。
- Pageの9改善は既知long_vertical2・N6・N2の1で、すべて既知fontの合成開発画像。独立実資料の追加改善とは主張しない。Nの一文字訂正は引き続き保留し、Oの政策は変更しなかった。
- P1の864 API出力はfresh通常版と本文/矩形/平均bitsが一致。実primary16,806とO part928は独立PILとbyte一致、PARSeq forward18,838試行/成功を記録した。P2は実cropのインク損失と損失なし誤認の双方を確認。P3は成立117中注釈98の内側BBox切断0、未注釈19は別扱いとした。
- Page経路の公開PNG回帰は隔離e5で末字「た」の欠落Red、変更していないOでeval/default・不正入力後再利用Green。本体へテスト/fixtureのみ接続し、実際に測定した推論sourceは凍結して保持した。
- 次はDEIM欠落時の方向fallback。注釈付き111ケース（6経路での重複回数）が水平band101..146個へ分割され、Horizontal認識に固定されていた。既存縦列検出を自動接続してよいとはまだ仮定せず、新しい方向選択の仕様・Red/Green/PBT・未測定対照と全Page/CLI gateを必要とする。改善全体は継続する。


## 2026-10-21: OのPage比較を撤回し実行物から再測定

- P1の実入力記録でOのPage採用を確認した一方、旧Page候補とbaselineのCLI/exampleバイナリSHAが同一だった。旧「444 ID全8経路不変」はOの検証になっておらず撤回する。旧本文/CSV/source/freezeは失敗履歴として保持する。
- `docs/vertical_page_artifact_revalidation.md`を先に固定した。73e9ab4の全Rust mtimeだけを更新しbyte不変を確認し、non-test lib/CLI/exampleのfresh=falseとbaselineと異なる二つのbinary SHAを確認した。既知positiveはfreshなuntraced PageとP1のtraceで一致した。
- baseline151入力のsourceはe5の90ファイルへ再結合でき、差はcfg(test)のnative.rsへの診断モジュール宣言だけだった。旧293再利用の別証明も維持する。全444候補の3model×2設定とCLI DEIM/bandsを新しいディレクトリへ再測定中。完了・公式/独立採点・無回帰確認までPage gateは未成立とする。
- 最新73e9ab4のCI39480288507は全5ジョブ成功。CI成功は旧Page比較の実行物取り違えを正当化しない。公開packetとPRも訂正する。改善施策の探索は継続する。


## 2026-10-21: 固定した完全prefix延長Oを本体へ接続する

- Nの342画像native比較は退行0だったが、新規7改善はすべて完全prefix延長だった。先行仕様`docs/vertical_uncertain_scalar_validation.md`で追加72画像を固定し、全414×3modelを検査した。新Auto216/Horizontal181 API・load5、全7,452文字/語比較で退行0だったが、新規の一文字訂正改善は0。事前条件に従いN全体を採用保留とし、既知4訂正を本体へ入れない。
- Oは`docs/vertical_strict_prefix_native.md`を実装前に固定し、Mのplans/combineを変更せず接続する。選択outputのT/C/charsetと実logitsのT、raw長T−1/T、原画素、異なる二分割の全文一致、元raw全文prefix、空白列/非空/非飽和/0.9..1 confidenceを要求する。旧H優先・同じ借用Session・既存sanitizeの位置を保つ。Lの支持はprivate候補だけで使い、旧公開splitterは維持する。
- 実Rust1,242採否と独立検算は一致。全414×3×8経路で9入力model組（既知2/N新規7）を改善、model30のnone/width/strictは文字誤り21・語誤り9減、50/100は不変。6,210の他5経路組とFragment単件/batch/専有batchの一致、完全文54・空GT診断114の不変を確認した。新規のnative測定は画像結果11,664/外側API8,760/load18で、旧baseline8,208結果の再利用と内部forward未計測nullを区別する。
- `docs/vertical_strict_prefix_page_gate.md`の444入力ID/443 unique RGB（旧paragraph2対照は同じRGB/GT）を前後8経路で比較した。旧293baselineのruntime同一性を3つのcfg(test)宣言差から検査して再利用し、他151baselineと全444candidateは新規測定。新Page API3,570・CLI1,190プロセスを保存した。全本文/行数/矩形/平均bitsは不変、21,312文字/語比較は退行0で、独立距離計算も全21,312 CSV行と一致した。Page精度改善とは主張しない。
- 通常124対象797成功/39ignored、ONNX lib549・CI対象integration45本213件・examples等24件、Python/OpenCV15件が通過。fmt/diffとClippyは新規警告0（default既存13/ONNX lib既存18）。実モデルCIには公開PNGの31/35字回復と既知一文字訂正の拒否、raw/通常保持・不正入力後再利用の回帰を追加する。
- `docs/benchmarks/vertical-partition-agreement/`へ、全414 PNG/RGB/GT/bbox、4,432 literal記録、全3,190 crop対応、採否/全native/Page結果、K/L/M/N/N2の反例・source・失敗履歴を保存した。別CWDで凍結sourceから再コンパイルし、モデル/フォント不要で実Rust1,242採否/plan・公式/独立スコアを再現した。reader15試験と内容アドレス履歴の全byte SHA検査も通過。weights/font/native binaryはGitへ入れず、Cargo.lockはarchive内snapshotだけでrootはignoredのまま。
- 本体でも先に公開API回帰をRedにした。コピーがmtimeを保持した初回はCargoが旧binaryを再利用し、同じSHA/欠落を再現した。失敗を残し、内容を変えずmtimeを更新して再ビルドし、本体ONNX lib549件が通過した。検証済み候補と14 Rust差分のbyte一致、H policyと依存lockの不変を確認した。全275 Rustファイルは検証済み候補とbyte同一で、本体の最終検査を追加した。正確headのremote CIもPR22で確認する。
- 目的全体は継続中。直接Fragmentで改善した新7画像も、DEIM cropを使うPage30には1..7文字の誤りが残る。次は実crop原画素とOのplan/棄却理由を先に記録し、crop依存の分割/支持を別仕様・Red/Green・新対照で検査する。


## 2026-10-21: 縦列の低信頼一文字Nの実モデル通過とnative接続検査

- `docs/vertical_uncertain_scalar_recovery.md` の固定案を変更せず、新Auto216/Horizontal284の計500画像API（load5、内部forwardは未計測null）を測定した。過去Auto810/Horizontal674は同じRGB・寸法・model・方向・decoder sourceであることを照合して再利用した。全新APIは成功し、literal範囲/停止/確率/mean bitsを検算した。
- 全342×3モデルの実plan120組を保存し、実Rustと別実装の全1,026採否/mean bitsが一致した。13本文組（既知6/N新規RGB7）を改善し、全3,078入力正規化組・6,156文字/語指標で退行0、非文字変更0、完全文36対照の誤変更0。native採用前の候補評価であり、公開本体はe5b11b4のままとする。
- `docs/vertical_uncertain_scalar_native.md` を接続前に固定した。output選択1 Red/5成功→6 Green、元RGB/trace/失敗callback2 Red/10成功→21 Green（既存prefix/uncertain testsを含む）。別native sourceでmodel30/50の実Fragment欠落を各Redにし、元raw/通常経路が基準と一致することも確認した。
- 同じ元logitsから必要時だけtop-1 traceを追加decodeし、追加の元NN/再load/再帰lockを避ける。静的output名/T/Cと実Tを確認し、H優先/既存sanitizerの位置を保つhookを別sourceへ接続した。実nativeの全13正例と不正入力後の再利用がGreen、raw/通常経路の本文とmean bitsも一致した。実native65 outer APIには意図した不正入力13件を含み、内部forward数とは区別する。本体へは未反映。

### 次のnative全件比較の先行仕様

- baselineはroot e5b11b4のsrc/test/Cargoを別コピーし、追加はcfg(test)の記録器だけとする。candidateは現在のnative sourceへ同じ記録器だけを追加する。全342画像×全3modelを除外せず比較する。
- 8経路はraw Session Auto/Horizontal、通常Pool Auto/Horizontal、Fragment Pool Auto/Horizontal、Fragment batch Auto、Fragment専有session batch Autoとする。各modelでraw/通常/Fragmentを1loadずつ使う。
- 6単件経路×342と2batch APIで、1variant/modelあたり外側API呼出しは2,054、画像結果は2,736。全3model/両variantの画像結果16,416を保存する。static batchの内部forwardやN chunk再認識数はこのAPI数から推定しない。
- 原RGB/model/charset/Cargo/source/SHAと方向を前後で固定する。GTを入力や採否へ渡さず、失敗もliteral errorで保存する。元Hを含むbaseline Fragmentを実測し、rawだけの値をH込み基準として扱わない。
- 変更を許すのはFragment Autoの3経路だけで、採用13組のliteral本文/meanは固定N結果、それ以外はbaselineの値を期待する。他5経路はbaselineと全件完全一致を必要とする。両variant内のFragment単件/batch/専有batchも全件一致させる。
- この比較、旧H/293ページ/既知縦書き7ページ/新対照API・CLIと必要suiteが終わるまで採用完了とはしない。反例・失敗・sourceとK/L/M/Nの公開再現packetもPR22へ保存する。目的全体を継続する。


## 2026-10-20: prefix回復Mの非採用と低信頼一文字の検査

- `docs/vertical_verified_prefix_recovery.md`を先行固定し、Lの支持をprivate候補内だけへ分離した。共有画素走査をbyte同一で抽出し、旧198画像の2,376幾何記録が完全一致した。支持15件とprefixの3 Red→9 Green、default library423件/36ignoredが通過した。
- 新72RGBを先に固定し、全270RGB/注釈162本文の6,480幾何記録を保存した。旧分割の134切断eventに対し新支持は0、26,533 raw chunkは独立PIL CCW/cropとbyte一致した。副spanを厳しくした12組の新split失敗は候補なしとして保ち、通常分割を変えない。
- 既存raw Auto162組をsource/RGB/model/charset一致で再利用し、新Auto648 APIを正常測定した。実容量近傍から84組がplanに進み、141 Horizontal結果を同一RGB/寸法/model/方向で再利用、追加529 APIも全件成功した。Mの新APIは計1,177、load15。内部forwardは未計測null。
- 実Rustの全810判定とmean bitsを別実装で照合した。全2,430入力正規化組/4,860文字・語比較の退行0、非文字変更0、完全文18対照の誤延長0。既知2画像だけがmodel30で各1文字を回復し、新72RGBの改善0で採用条件未達だった。Mは本体へ未接続で、native/Page/CLI採用検査は未実施。
- scan抽出時にproptest内部のfloat sampler assertionが発生した。再現stream試験では同assertionを再現できず、別コピーのcfg(test) strategyだけ整数bit生成へ変更し元propertyを保った。自分で追加した無entropy stream試験のhangは対象PIDだけ終了し失敗source/logを保持、境界の直積試験へ直して7件成功した。製品処理や依存versionは変更していない。
- 全source/入力/GT/確率/失敗/正式9表とSHAは `tmp/vertical-verified-prefix/` に保持する。K/L/Mの公開パケットは後続と合わせる。公開本体はe5b11b4のまま、目的全体を継続する。
- 次は `docs/vertical_uncertain_scalar_recovery.md` のN。容量近傍の低信頼な1文字を、異なる二分割の文字別高信頼一致で検証する。Mを後から緩めず別候補としてtest-firstで始め、新72RGBを別途固定する。
- Nは別コピーの5 Red/4成功→9 Green（64例PBT）、default library432件/39ignoredが成功した。実literal traceの保存再生810組で既知6本文組を改善、全4,860文字・語指標の退行0/非文字変更0/完全文18対照の誤変更0だった。Nの新NNは0で、M新72もNにとって既知回帰とする。N新72RGB（54本文/18図形）をGT/描画条件/SHA付きで生成前固定し、旧270との重複0を確認、合計342RGB/注釈216本文が揃った。新N72の1,728幾何記録と旧270の6,480記録で計8,208を揃え、支持字形の切断0を確認した。新7,395 cropも独立PILとbyte一致した。新Auto216 API（load3/内部forward未計測null）は全件成功し、全1,026元結果を検算した。実planは120組。追加Horizontal認識と新N72の採用検査、native/Page/CLIは未実施で、本体へ未接続。


## 縦列境界K/Lの不採用と合意付きprefix回復の継続

- 先行仕様`docs/vertical_composite_boundary_proposal.md`のKは別sourceコピーで3失敗/3成功→6成功（64例PBT）へ進めた。新36RGBを含む全150RGBで、切断116→36イベントに減ったが7原RGBで残り、候補全体をNN前に棄却した。default library406件と既存縦組み幾何33件成功。製品sourceは不変。
- 次のLは`docs/vertical_advance_boundary_proposal.md`を先行固定し、隣接anchorの送り幅と独立細線の保護を別コピーで検査した。新6テスト4失敗/2成功→全12成功（2つの64例PBT）。default library412件/36ignored・縦組み幾何33件成功。
- Lの新48RGB（変動字間本文36/図形12）と既知150RGBを合わせ、全198RGB/注釈108本文を固定した。全2,376幾何組で切断124→0、新分割エラー0。旧baseline1,800組の再利用を新規実行に数えず、新baseline576と候補2,376を収録。17,372 RGB chunkは独立PIL回転/cropとbyte一致した。
- raw Horizontal候補APIは各model732・計2,196回、全件正常。実入力RGB/寸法と全step/平均bitsを独立照合した。全1,188本文・594入力model組・3正規化の文字/語3,564比較と公式9 CSV/MDを保存した。両側の分割失敗111組はraw nullのまま残し、宣言済み空proxyでのみ採点した。新規/解消失敗0。
- Lは認識18入力model組（正規化別42行、strictでも6組）に退行があり不採用。none文字誤り総数9,322→9,322、語1,030→1,049で、幾何成功を精度改善とは数えない。負例出力変更0。条件未達のため製品接続・後段Page/CLI A/Bを実行済みとはしない。
- 仕様追記と比較元driverが重なったSHA検証失敗、容量対策の検証済み復元cache整理、全source/GT/raw/ZIP/失敗は`tmp/vertical-composite-capacity/`と`tmp/vertical-advance-capacity/`に保持する。公開パケットは後続と合わせる。現在公開e5b11b4のCI39423666923は全5ジョブ成功、正確なPR本文も確認した。
- 次は`docs/vertical_verified_prefix_recovery.md`。Lを旧公開splitterへ一律適用せず、新しいFragment+Auto回復候補内だけで安全な支持を使う。実output metadataのT近傍で、元raw全文を保持し二分割の全文一致/空白列一致を満たした延長だけを調べる。完全な30/50/100文字本文を含む未生成72RGBを先に固定する。目的全体は継続する。

## 2026-10-20: 縦列prefix延長候補の事前棄却

- `docs/vertical_prefix_capacity_proposal.md`へ、元raw prefixを保持し異なる二分割の全文一致を要求するtest-only候補を先行固定した。Hの既存条件は変更しない。小さい結合/幾何Red→Greenと64例PBTを先行し、純粋6テストが成功した。
- 新36本文/24図形を先行固定し、既知54RGBも残した。575個の旧index/fixtureからの312,538 SHAと新RGB/PNGは未重複。既知3書体の合成開発対照で、独立holdoutとは数えない。
- 文字数gateだけdummyで開いた全114×3形状342組の幾何検査で、5元RGB・12個のmodel/route/glyph切断（両chunk端のイベント24）を検出。固定済み全件gateに従い候補全体を追加NN前に棄却した。モデルload/Session.run/baseline/chunk APIはすべて0。実rawに対する到達数や精度改善ではない。
- 全1,151 chunkのRGBは独立PIL CCW回転/cropとbyte一致。342行CSVと24切断イベントも独立再現した。default library400件成功/35ignored、新規Clippy警告0。`docs/benchmarks/vertical-prefix-capacity/`へ全入力/GT/glyph bbox/source/失敗を保存する。
- 製品経路は変更していない。分割境界の離れた画線保護を別の仕様と未認識対照で調べる。目的全体は継続する。

## 2026-10-20: 縦列のraw停止理由と出力容量の監査

- `docs/vertical_output_capacity_audit.md`をメタデータ/追加NN前に先行固定し、静的上限抽出をRed→Greenと64例PBTで検査した。実モデル3個の出力はT=31/51/101。画像を渡さずmetadataを読み、Session.runは0回。
- H全42RGBと既知7ページの全12 DEIM cropを結果で選別せず固定し、raw候補API162呼出しを記録。EOSは27/38/36、低confidenceは27/16/18、全frame消費は0。余白/図形も最終frameで停止し得るため、上限付近という条件だけで回復を採用しない。
- 旧raw126組は本文/f32平均bits完全一致、Page36組の平均bitsも一致。162組のstep/range/停止条件とf32逐次平均を独立照合した。Page本文との差は30 numeralsの末尾space1個だけで、既存後処理の差だった。
- `docs/benchmarks/vertical-output-capacity/`に全raw/GT/crop/metadata/source/失敗ログを保存。空の復元先でモデルなしに162行CSVとanalysis.jsonのbyte一致を再現した。静的上限3・候補5・準備3・復元2テスト成功、新規Clippy警告0。
- 製品認識処理は変更していない。既知データの原因監査であり、新規holdoutや精度改善とは数えない。既存縦列分割の安全なページ経路への接続、crop境界や紙の損失を引き続き調べる。目的全体は継続する。

## 2026-10-20: 検出された長い縦列の分割回復

- 先行仕様 `docs/vertical_model_capacity_recovery.md`。色背景準備だけのH1は9列すべて支持group40未満で、追加NN前に棄却した。H2は元RGBを保ち、支持用maskだけへ有理数Otsuを適用する。小さいRed→GreenとPBTを先行し、数値条件は追加NN前に固定した。
- 未使用のNDL3コマの本文9列・余白6・図形18を認識前に固定。旧/追加全42画像×3モデルの候補で、raw Horizontal chunk API81回を記録した。旧コマ5第2列と追加コマ6第1列の30モデルだけを採用し、旧9列269→236、新9列209→177の文字誤りへ改善。全入力別none/width/strict文字・単語指標は非悪化、余白/図形72組は出力同一。モデル/方向/同一書籍を独立標本とは数えない。
- 実モデルRed→Greenで、既存Fragment+Autoのraw結果の直後だけへ接続した。元raw空白列が異なる候補は棄却し、同じ借用sessionへ元RGB chunkをraw Horizontalで渡す。通常SingleLineと明示Horizontalは従来どおりで、採用後も既存Fragment後処理を使う。
- 既存287画像＋実本文6抜粋の293入力を前後8経路で新規実行。7,032文字/単語比較に退行0、本文変更は30モデル2抜粋×2設定の4組だけ。行数/矩形は全件同一、confidence bit変更も4組だけ。別の既知縦書き7入力も前後8経路・168比較で全本文/metadata同一。API3,516回とCLI1,172プロセスを内部forward数と混同しない。
- 製品sourceの別コピーで実API630組の本文/f32 bitを全照合し、余白/図形216 Fragment経路組も従来値を保持した。全42maskは独立有理数計算と全byte一致、モデル不要の実Rust replayは126判定・81保存callbackを再現した。通常762件・ONNXスイート718件・example24件成功、新規Clippy警告0。
- `docs/benchmarks/vertical-capacity/` に全入力/raw/棄却/失敗/source/採点表を保存。空の復元先で48表・33,528 CSV行と、モデルなしのRust再コンパイル/126判定を再現した。初回のCargo作業ディレクトリ起因の相対パス不具合も履歴を保って修正した。
- 同じ書籍/既知書体の開発対照で、学習との重複は不明。縦書き7入力の30モデルには欠落が残り、出力容量・分割境界・空白不一致などの改善を続ける。目的全体は継続する。

## 2026-10-20: 縦組み実スキャンの初回残差監査

- 先行仕様docs/real_vertical_scan_audit.md。NDLのPDM表示を確認した『坊つちやん』（PID1884801）の3コマから、各冒頭本文3列をNN前に選び、元JPEG/crop/RGB/旧字の手転記を固定した。全見開き評価ではなく、9列・列間LF込み392 scalarの開発対照である。ルビは画像に残し、本文GTから除く規約を明示した。モデル学習との重複は不明。
- 全24ページ本文組を取得。none/width文字距離はCLI DEIM=13、bands=380、PageRecognizer30=269、50=14、100=13（eval/default各）。全モデルの9矩形は同一だった。製品変更や新たな精度改善はまだない。
- 全9検出列の8単行経路×3モデル、計216組を追加取得した。raw/ページ27組は全mean bit一致・25本文byte同一で、残る2本文は末尾ASCII空白1個だけの差だった。主な欠落がrawに存在することを切り分けた。
- docs/benchmarks/real-vertical-scan/ に全原画像/真値/source/raw/矩形/confidence/API/CLI/正式表と、取得・crop修正・パッケージ失敗履歴を保存した。別の空ディレクトリでモデル不要の12 CSV/Markdownと27対応の再現が成功した。ページ/API呼出し数を内部NN forward数と同一視しない。
- DEIM縦列は既存の明示縦列分割へ接続されておらず、入力容量が欠落の原因候補である。次のtest-only支持案をdocs/vertical_model_capacity_recovery.mdへ先行固定した。元RGBを認識へ使い、支持用maskと既存分割を先に純粋テストする。分割による回復は未測定で、目的全体を継続する。

## 2026-10-20: 点付き英字の構造支持

- 先行仕様はdocs/latin_dotted_body_kernel.md / docs/latin_dotted_body_policy.md / docs/latin_dotted_body_controls.md。上部の点をi/jのstemへ一意に所属させ、全前景・本文位置・round profileを確認し、モデルが既に読んだfragmentを最後の単行fallbackだけで保持する。単語/font照合・文字置換・追加NNはない。
- 純粋30テスト、4つの64例PBT、実画像、実Session/Poolの3モデルRed→Greenを先行。追加NN前にowner/policyの数値を固定し、旧223,143組と新59,322 raw forwardを比較した。全282,465組のnone/width/strictで入力別文字・単語退行0。102変更組でnone完全一致45、width完全一致91を回復し、strictは同点だった。
- 102組は23種類の元RGB、追加分は未重複19種類。既存1組などには字種誤読が残る。旧2,871画像×3モデルのraw正読後の損失は467→456。既知3書体・6本文の開発対照で、未知書体/資料の改善率や独立標本数とは扱わない。
- 全19,440条件をNN前に固定し、1,854独立再描画・126裸本文の成分照合・4,536図形PNGの再構成と108条件の事前目視を確認した。実NN入力19,774件と元Auto/effective Horizontalの3形状tensor bitも一致した。図形条件のbody-only真値は診断上の規約であり、意味が一意な非文字GTとはしない。
- 実製品282,465保存組、実API426組、CLI426件が保存期待値と一致した。CLIにHorizontal指定がないため、方向指定26組はPool APIで検査し、CLIでは同じ元画像のAuto26組を別期待値で検査した。通常740件、実モデルCI対象692件、example等24件が成功し、新規Clippy警告0。
- docs/benchmarks/latin-dotted-body/ の空ディレクトリへの独立復元で153 CSV/153 Markdown・711群集計、全画像/raw/確率/API/CLIを再照合した。復元sourceの新規default buildでもRust282,465組と19,440×3形状tensor bit照合が成功した。親Cの全履歴は今回再実行せず、公開archiveと既存成功receiptをSHAで固定している。
- 今回の追加ページ精度測定は0で、実モデル回帰の既存ページNNは実行済み。NN/ページ断片経路は比較元とbyte同一。実資料の残差、Latin以外・長い本文・複数closer・真縦組みなどを引き続き調べる。目的全体は継続する。

## 2026-10-20: Latin追加対照の保存方向比較

- 先行仕様 docs/latin_auto_orientation_saved_audit.md。既知横組みのAuto回転636元画像を全3モデルの保存rawで対にした。追加NNは0、製品処理の変更もない。
- 元RGB/真値/原画像のSHAと両方向の実RGBを検査し、画素回転の4テストを既存pairing4テストへ加えてRed→Greenを確認した。
- 生文字距離6,608→4,446、単語距離2,136→2,073、生完全一致42組回復だが、100モデルで文字16組・単語6組が悪化した。textだけにも退行があり、一律Horizontalは採用しない。strictの文字退行も2組ある。
- docs/benchmarks/latin-orientation-saved/ に全27群/46退行指標行と正式18 CSV/MDを保存した。親packetから別の空ディレクトリへ画素/rawを読み直し、モデル不要で全表と全集計/反例を再現した。
- 既知fontの開発対照で、方向識別の汎化や新しい製品改善とは扱わない。真縦組みを含む方向対照と、残る部品/closerの後処理損失の改善を継続する。


## 2026-10-20: 接触した英字の構造支持

- 先行仕様は docs/latin_pair_bridge_kernel.md / docs/latin_pair_bridge_policy.md / docs/latin_pair_bridge_controls.md。元RGBを変えず、1画素接触の不均等な2 view・全前景所有・marker profileで、モデルが既に読んだ丸括弧を単行の最後のfallbackだけで保持する。単語・文書種別の置換や追加NNはない。
- Red→Green/PBTを先行し数値条件を凍結。旧162,915組と追加60,228 raw forwardを比較し、112組でnone/width完全一致を回復した。全群・全正規化の入力別文字/単語退行0、strictは同点。112組は48種類の元RGB、追加94組は未重複42種類で、未知資料の改善率とは扱わない。
- 実製品223,143組再生、公開API9,184組とCLI9,184件が一致。通常710件・実モデルCI対象657件・example等24件が成功し、新規Clippy警告0。ページ断片とNN経路は不変で、新規ページ精度測定は行っていない。
- docs/benchmarks/latin-pair-bridge/ のモデル不要再現で親評価、1,188 CSV/MD、3監査、全画素/raw/mean/API/CLIを再照合した。復元sourceの新規default buildによるRust223,143組と19,440×3形状tensor照合も成功した。
- Latin以外、長い本文、複数closer、真縦組みを含むAuto方向などの改善を継続する。

## 2026-10-20: 狭い丸括弧の元画素profile

- 先行仕様は `docs/narrow_round_marker_profile.md` / `docs/narrow_round_marker_body.md`。幅・画線連続性・上下端・中央帯の例示とPBTをRed→Greenで固定し、本文の既存所有関係を共有した。文書・単語の置換は追加しない。
- 条件を凍結して旧137,991組と追加24,924 raw forwardを比較し、62モデル/条件/方向組でnone/width完全一致を回復。全群・全正規化の入力別文字/単語退行0。変わった元RGBは19種類、追加で変わる未重複RGBは6種類で、独立標本や未知資料の改善率とは扱わない。
- 公開APIの3モデルRedを先行し、最後の単行fallbackだけに接続。実製品162,915保存raw再生、公開API8,564組、CLI3,500件が凍結案と一致した。通常685件・ONNX対象627件・example23件が成功。
- 新規Clippy警告をテスト内の2つの反復呼出しだけで解消し、境界テスト・全13profileテスト・Clippyを再確認。計測snapshotと最終sourceのruntime部分はbyte同一である。
- 新規ページ精度測定は行っていない。Latinのx-height、長い本文、複数closer、真縦組みを含む残差の改善を続ける。

## 目的
- `ndlocr/src/ocr.py` を中心に Rust 移植する。
- ONNX Runtime で CPU 推論を先に安定化する。
- 出力互換（xml/json/txt）を維持する。

## フェーズ
- Phase 1: CLI土台 + 推論I/O（前後処理、charset読込、smoke）
- Phase 2: `DEIM -> crop -> PARSEQ` 接続、カスケード、中間構造
- Phase 3: 出力互換（JSON/XML/TXT）
- Phase 4: 読み順整序
- Phase 5: 回帰・性能・配布

## テスト方針
- test/document first
- t_wada方式（Red->Green->Refactor）
- 効果的な箇所で property-based testing

## 2026-10-18: 全角ASCIIに隠れる字形退行

- `docs/width-sensitive-evaluation.md` と5テストを先に固定し、4失敗を確認。
- `eval_ocr_metrics.py --normalize width` は全角ASCII94文字だけを写像し、空白・大小文字・他の互換文字を保持する。既存の指標と併記する補助採点で、認識本文には適用しない。
- 標準ライブラリのCLI/全範囲/既存互換テストを3OSのCIで実行する。モデル30/50/100の同一chunk比較はomo #79で行う。

## 2026-10-18: 外周近白紙

- 仕様 `docs/nearwhite-panel-paper.md`、局所Red 2件と後段色縁処理のRed 1件を先に固定。
- 全成分250以上の4近傍経路を保護し、既存色縁除去でも元RGBを維持する。囲まれた近白字形と249境界を検査。
- 固定28画像の全文比較と公開pin統合はomo #79で実施。全画像の精度改善・JPEG一般化・production readyは未確認。

## 2026-10-18: 左右の補間色外縁（検証中）

- `docs/lateral-panel-fringe.md` を先に固定し、左右階段縁の例示/PBTと孤立色文字による追跡中断の Red を確認。
- 混合色・外周紙・長い連続境界で制限した最小実装で局所8テストがGreen。罫線・厚み・長さ・囲まれた字形・紙RGBを負例で検査。
- 実画像+2度は現時点で14行のまま。固定28画像の認識・速度比較と公開pin統合は未実施。production ready・16行回復は未達。

## 2026-10-18: 切り出し境界の連結成分（検証中）

- `docs/clipped-band-components.md` と失敗する境界回復・不正入力テストを先に追加し、明示呼出しの汎用APIでGreen化。
- 四辺・斜め連結・独立図形・接続罫線・複数帯・新規矩形衝突を例示と独立探索oracleのPBTで検査（9テスト）。
- 既定検出経路は変更しない。固定80画像の画素保存と28画像の全文認識をomo #79で比較中。離れて欠落した文字・同形図形の意味分類・production readyは保証しない。

## 2026-10-18: 原chunkの余白感度

- `docs/margin_sensitivity.md` と原RGB fixture欠落のRedを先行し、製品の全ページ出力を再現して6件を取得。非白RGB保持と前処理tensorの変化を検査する。
- 右側の白列追加probeもRed→最小実装→Green。通常fixture1件・probe3件、全60条件×2推論の再現性を検証し、Ubuntu/macOSの実モデルCIにも同じprobeを追加。
- 白2列で正読 `and culture.` が `Cand culture.` になる反例を固定。既知6件の信頼度選択0誤りは事後分析であり、精度改善・未知資料への汎化・本番採用を意味しない。次は選択条件を先に固定した別資料の全件非悪化評価。

## 2026-10-18: 原/縮小候補の信頼度比較（明示opt-in）

- `docs/confidence_margin_candidates.md` を先に固定し、7テスト中6件のRedを確認。有限信頼度比較PBT・不正RGB・推論失敗・同点・同一入力の1呼出しを7件Green化。
- `infer::margin_candidates::recognize_horizontal_margin_candidates` は原画像を先に認識し、幅/高さ5倍以上のchunkだけ縮小限定余白を比較する。本文は変更せず、平均信頼度が厳密に高い場合だけ候補を選ぶ。
- 通常pipelineは変更しない。未知資料6画像+既知8画像の製品接続検証・追加推論時間の測定はomo #79で実施する。精度改善・本番採用は未確認。

## 2026-10-18: 信頼度選択の追加回帰監査

- 14画像の生の合計827→819に対し、全角ASCIIだけを折り畳む補助評価では770→763。合計改善でも既知2chunkの `AI` → `Al` が残り、画像別/既知字形の非悪化条件は失敗した。
- `selected()` のRustdocと比較仕様へ実測した制約を追記。通常pipelineと選択処理は変更せず、自動選択の製品採用は保留する。
- 次は同じ原chunkを公式30/50/100文字モデルで比較し、分割条件を分離する。既知14画像は開発用回帰として扱い、未知資料での精度改善とは称さない。

## 2026-10-18: 候補解析の実入力capture

- `docs/candidate_input_capture.md` と未実装Red4件を先行し、任意RGB往復PBTを含む4件をGreen化。候補exportへ `--chunk-input-dir` を追加し、同じ認識直前RGBを可逆PNGへ保存する。通常pipelineは変更しない。
- omo #79の既知正読対照3帯/226字、30/50/100モデル計49区間で、過去の本文・区間・寸法を再現。同じ区間の保存RGBはモデル間でも同一。50の引用符置換は正解が第2候補、50の括弧欠落1件と100の引用符欠落2件は固定位置の置換だけでは復元できない。
- 生の誤り0/2/2に対し上位10候補oracleは0/1/2。これは原文を参照した上限で、実装済み補正器の改善ではない。画像には対象記号が残るが、モデル重みと内部resizeの影響は未分離。本番モデル・製品pinは維持する。
- exampleの4テスト・通常Clippy成功。厳格Clippyは未変更の本体9警告で失敗し、ログをomo側へ保存。次は独立資料の小記号/色/書式/解像度の対照を先に固定し、欠落を含む認識側改善を検証する。

## 2026-10-18: 面積平均縮小の比較診断

- `docs/area_resampling_probe.md` と細線の未実装Redを先行。最小Green後に分数境界・BGR・拡大互換・入力拒否・濃度保存PBTの4 Redを確認し、5件Green化。truth/重複ID/不正座標拒否のRedを追加し、example全6件Green。
- `compare_parseq_resampling` は横書き同一RGBに既存bilinearと縮小軸の面積平均を適用し、同一CPU sessionで実行順を交互にする。全chunkの本文・信頼度・寸法・区間・前処理/推論時間・失敗を保存する。新規公開APIや製品pipelineの変更はない。
- Ubuntu/macOSの実モデルCIへ6テストと既存candidate exportとのbaseline本文一致smokeを追加。omo #79で既知14画像、事前固定した合成36行、記号退行3帯×3モデルを比較する。合成対照は未知実文書holdoutではなく、実験だけで精度向上・本番採用を主張しない。

## 2026-10-18: 本流 uint8 OpenCV 前処理の比較診断

- `docs/opencv_preprocessing_probe.md` と通信境界の未実装 Red を先行。
- 診断専用 Python oracle を接続し、既存 bilinear と同一 RGB・同一 session で比較する。製品 Rust・公開 API・通常依存は変更しない。
- 不正入力・途中 EOF・応答長・非有限値を検査し、独立した6種類の画素 fixture で BGR/NCHW と丸めを照合。Ubuntu/macOS CI で実モデル経路も検査する。
- omo #79 の既知14画像・既存合成36行・記号3帯に対し、推論前に固定した全件非悪化条件で評価する。未知実文書 holdout と pure Rust 実装は別途必要。

## 2026-10-18: 近接した太字2行の検出

- `docs/narrow-line-gaps.md` と実資料/PBTの2失敗を先行。1pxの谷と少数文字の接触を、通常高・本文高・幅で限定して分離する最小変更。局所3件・既定lib136件・ONNX lib145件成功。通常Clippy成功（既存9警告、変更箇所なし）。
- 気象庁の見出し2行をnaive/fastとも分離し、縦画で結ばれた単一字形・探索上限の対照を保持。実資料は初回観測後の開発用回帰。
- omoの公開pin統合は保留。既知14画像と新規診断5画像で全文/正読対照を比較し、悪化がないことと見出し欠落の改善を要求する。未知資料への一般化・production readyは未確認。

## 2026-10-18: 行頭小記号の本文文脈を保持

- `docs/leading-symbol-context.md` と合成例/実画像/容量境界PBTの3失敗を先行。先頭の小さな単一成分と本文を、実モデル容量・既存字形数上限内でのみ結合。普通の文字、複数成分、非隣接や本文不足は変更しない。文字置換は追加しない。
- 局所4件・既定lib140件・ONNX lib149件成功。通常Clippyは成功（既存9警告）、fmt/diff確認。omoの同一JMA資料2行の初期診断では、1行は丸点を正読し、他方は中点の誤りが残る。
- 公開候補pinを使い、omo #79の19既知入力を画像ごとに生/正規化距離で比較する。全ページ合計での退行相殺は認めず、採用は全ゲートが通ってから。未知資料・production readyは未確認。

## 2026-10-18: PR #21 引き継ぎ・下線付き長行

- 先行仕様は `docs/accuracy-followup.md`。神戸市お知らせの正解を先に固定し、URL4行の分割拒否を再現。合成/実画像/PBT 3 Red→Green、既存成功経路とstrict APIは保持。
- 通常分割失敗時だけ、下部の長い細線を判定画像から除外し、元RGBを覆う区間を返す。全画素を保持。
- 診断fallback基準との全文比較は生210→82/1034字、width161→33/1034字。先頭28行同一、URL4行は各行改善。URL underscoreなどは未解決で、生WER35→38/41の悪化も記録。未知様式/製品全体/性能の改善は未確認。
- 通常467件、実ONNX362件が成功（追加ガードと実URL回帰は別途成功）。次は囲み枠の行頭誤検出、URL残差、別様式の小記号を分離して検証する。

## 2026-10-18: 離れた囲み側線を本文から分離

- `docs/accuracy-followup.md` の先行仕様・実画像/合成/PBT 3 Red→4 Green。全行trim案は正読中点を落としたため棄却し、近い枠を保持するRed→Greenで対象を限定。
- 複数行をまたぐ左右の細線と十分離れた本文を確認した行だけ左右を縮め、RGBと上下座標は不変。新規資料で本文変更は1行だけ、生82→76/1034字、width33→27/1034字。下線改善との合計で210→76、161→27。
- 通常473件・実モデル373件と別診断2実行が成功。下線除去はURL4行合計10→3でも正読1行を壊すため採用しない。未知様式、速度、DEIM全体への効果は未確認。

## 2026-10-18: 強い2行に覆われる余分なDEIM領域

- 先行仕様 `docs/deim_duplicate_extent.md`。実画像1 Red、合成/cascade/PBT 3 Red→Green。
- 既存の異クラスIoU選択後、幾何・2倍超の信頼度・非白画素被覆を確認した弱い複数行候補だけを除く。生DEIMと勝者metadataは不変。CLI/cascade/PageRecognizerで共用。
- 固定4画像×2設定の全件評価で既定CLIの神戸市だけ改善（生111→75/1034字、width64→28字）。他7組は本文byte同一。別様式のNDL公開資料を初回対照に追加し、以後は開発回帰と扱う。
- 通常479件・実モデル381件成功。境界/近白画線/不正入力/連鎖保護を検査。JPEG/低解像度と速度は未検証。次は改行をまたぐ括弧と既存sanitizerによる字形脱落を診断する。

## 2026-10-19: ページ内の行断片の閉じ括弧

- 先行仕様 `docs/page_bracket_context.md`、単体API未実装Red→3件Green、実CLIのRed→Green。
- 生PARSeqが持つ閉じ括弧を、ページ内の行の不均衡だけで削除しない。CLI/PageRecognizerの両構築経路で断片向け処理を使い、既存単行API・生APIは維持。新しい文書固有置換はない。
- 4資料の比較後、処理方針を固定して気候変動資料を初回対照へ追加。5資料×2設定の全30文字指標が非悪化。変更した6組は閉じ括弧の保持だけ、他4組の本文は同一。
- 神戸市DEIMは生75→74/1034字、width28→26。NDLは生60→60/1058字、width55→53。新規気候変動は生61→60/800字。通常482件、ONNX対象387件、追加互換/並列5件（再実行を含む）が成功。
- 同一targetでfeature違いのビルドを並行させた検証失敗を分離し、順番に再実行して成功を確認。性能・全ページの総推論数は未計測。次はNDL本文先頭「一」の欠落を入力/crop/モデル/後処理へ分離する。

## 2026-10-19: DEIMが途中で切った行端の短い横画線

- 先行仕様 `docs/deim_edge_glyphs.md`。生DEIMとrawモデルで「一」の左17列の欠損を確認。CLI Red、合成/cascade/PBT 3 Red→Green。
- 左右境界の単一成分を局所窓で完結させ、短い中央の横画線だけ認識領域へ戻す。元RGB/上下座標/生検出/信頼度/文字数は不変。別成分、長線、背の高い字形、原/提案の矩形衝突を保護する。
- NDL資料で生60→59/1058字、他9組の本文同一。元5資料の全30指標非悪化。
- 事前固定したJPEG90/75・半分PNGの15変形も全90指標非悪化。NDLのJPEG2条件だけ1/2文字改善、他28組は同一。実スキャン/未知資料の一般化は未確認。
- 通常486件＋追加境界1件、ONNX395件、回収を使うページ並列1件が成功。次は残る全幅の境界切りや非横画線の欠損を分離し、既存opt-in成分回復との比較を検討する。

## 2026-10-19: 四辺成分回復の既定化を棄却

- 先行仕様 `docs/deim_component_probe.md`。既存opt-in APIを固定20画像の全368領域で比較し、219領域・50/100両モデル438対を記録。
- ロゴ正読、ビ/ピ、小数点に退行があり、信頼度上昇中の誤読も確認。既定化を棄却し、製品経路は不変。
- 短い横書き「で上昇」はcrop/自動回転でなく30文字モデルの生逆順と判明。50/100両モデルは正読するため、別仕様で慎重な合意条件を検討する。

## 2026-10-19: 短い横書きの順序を2モデル合意で再確認

- 先行仕様 `docs/short_horizontal_line.md`。生30モデルの「昇上で」を50/100の正読から分離し、CLI Redと方針の未実装Red→Greenを確認。
- CLIの30-bucket・2〜4文字・低信頼度・狭い横書きだけ、両モデルの高信頼度一致と元の文字multiset保持を要求する。候補の出力そのものを選び、字句置換やtruth条件は加えない。失敗時は元結果を保持。
- 固定20画像×2設定の全120文字指標が非悪化。気候資料の原/JPEG90が各2文字改善、他36本文はbyte同一。96合成cropも非悪化（本文変更0）。
- 通常494件、ONNX対象404件、追加の設定互換1件（CLI2設定）が成功。fmt/diff/対象Clippy成功（既存17警告）。追加推論の負荷増はあり、未知実資料/速度改善は未確認。
- 次はURLの途中脱落・句読点とモデル文字集合外の残差を切り分け、字句固有の補正なしで改善できる入力/候補条件を検討する。

## 2026-10-19: URL/パスの行末literal hyphenを保持

- 先行仕様 `docs/url_literal_hyphens.md`。構造処理無効の本文と対比し、英文ハイフネーション処理がURLのhyphenを消していると分離。単体3 Red→5 Green、実CLI Red→Green。
- 末尾tokenにURL/パスの見える構造がある場合だけ二行とhyphenを保持。URLの後ろの通常英文の結合、モデル出力・入力RGBは維持。資料固有文字列を使う条件はない。
- 固定20画像×2検出器×前後=80プロセスが成功し、全120文字指標は非悪化。7組でURL2本を保持し、他33本文は同一。神戸市DEIMは生74→70/1034字、width26→22。構造処理無効の本文も同一。
- 通常499件、ONNX対象411件、fmt/diff/対象Clippy成功（既存17警告）。前版5a8dad9の全5CIも成功。ドメイン内hyphen/underscoreのモデル脱落は未解決で、次は100文字cropの安全な区間分割を診断する。

## 2026-10-19: 全幅100文字入力とchunk文脈の3案を棄却

- 先行仕様 `docs/page_chunk_probe.md`。固定20画像×2検出の730cropで、細分割・実入力幅までの結合・全100モデルへの置換を診断。比較元40全文は実CLIの明示設定で再現し、全てbyte一致。
- 各120文字指標中63/30/15で退行があり、3案の一律既定化を棄却。改善もあるが局所/単独指標だけでは採用しない。製品コードの変更なし。
- 結合helperは診断専用で、未実装Red→境界/不正入力/非連続白余白/列被覆PBT等5件Green。診断2設定・40 CLIが成功。fmt/diff/対象Clippy成功（既存17警告）。前版dde5d3aの全5CIも成功。
- 現40ページの全幅100認識は最大77文字で、既定の80文字以上の長行再分割は未検証。次は固定真値を先に用意し、既存の中央分割が字形を切る誤りを別に診断する。

## 2026-10-19: 同クラスの近似同一DEIM領域

- 先行仕様 `docs/deim_same_class_duplicates.md`。26個の固定長短行対照で同一物理行の3 queryを再現し、単体/PBT 2 Red→3 Green。既存90% IoU選択をクラスによらず適用し、生DEIMと勝者metadataは維持。
- 長行の中央処理は変更しない独立binaryで比較。新26+既存20画像×2設定の全276文字指標が非悪化。新規DEIM4組で生116→18、244→105、242→17、129→7。他88本文、うち既存20画像40本文は同一。
- 通常507件と実DEIM含む4件が成功。fmt/diff/対象Clippy成功（既存17警告）。認識confidenceを検出scoreと取り違えた初回テストを訂正し、全DEIM scoreと勝者metadataを別に記録。
- 新26の前版52 CLI＋候補92 CLIを実行し、既存比較元40本文は同一binary/image SHAを確認して凍結再利用。長行の境界変更はこの独立版から再評価する。

## 2026-10-19: 長行再分割の画線と単語空白

- 先行仕様 `docs/long_line_split.md`。26入力を推論前に凍結し、80文字条件を通る14入力のうち12で元中央の連結画線切断を再現。幾何の未実装Red→9 Green、安全な原境界を保持する追加2 Red→計11 Green。下線の判断だけを共有し、認識RGBは保持。
- 元中央が既に連結画線を切っていなければ保持し、それ以外は±行高の近い白列を使う。新たな移動では内部字形の狭い隙間を避け、実単語空白を左右の間へ戻す。cached/legacyで同じ方針。
- 初回4指標、修正後に重複領域だけの1指標の退行を記録。同クラス重複抑制を独立検証してから、その版8edefcdを比較元として長行変更だけを再比較した。全46画像×2設定の全276文字指標は非悪化、15本文改善・他77同一、うち既存20画像40本文は同一。
- 通常518件、ONNX対象435件が成功。実段落cropと下線URIはbandsで完全一致、原の画線切断12→0。fmt/diff/対象Clippy成功（既存17警告）。直前版の比較元92本文はSHAを照合して再利用し、候補92プロセスを実行。
- 和文の4変形は初回100モデルで2〜8文字に崩れ、80文字条件に達しない。次はこの入力密度/early-stopの原因を分離する。未知資料の汎化/高速化は未確認。

## 2026-10-19: 高密度長行の短すぎる認識を同モデルで再確認

- 先行仕様 `docs/dense_line_probe.md` / `docs/dense_line_recovery.md`。120字和文4例の生100モデルはEOSでなく確率0.35未満で2〜8字に停止。全幅RGBを既存chunkへ渡す診断では30/50/100全てが4例完全一致。
- 幾何・前景列/支持列群と短い初回結果で対象を限定し、同じモデルのHorizontal部分認識が全て高信頼度で十分な文字量/種類を説明するときだけ採用。CLIは既存100poolでsplit無効を尊重、PageRecognizerは供給primary poolを使う。単行/生API・Auto・元画素・検出枠を保持。
- 新33対照を処理前に固定して既存46と全件比較。初回は重複領域2画像のDEIM/runtime100で各2文字退行（12指標）。不採用案の全出力/採点を保存し、小さい領域面積の50%以上重なる領域では追加認識しない条件をRed→Greenで共有した。
- 修正版は79画像×4経路×3正規化の全948文字距離が非悪化。65本文改善、他251本文はbyte同一。CLIの和文2例はbandsで117→0、119→0/120字。既存20画像のCLI40本文は同一、PageRecognizer30の既存資料7変形も改善。
- 空白区切りの単語指標は6組のnone/widthに退行があり、runtime100の生総単語誤りは690→691/997語。文字距離の採用ゲートと分離して全件保存。全幅ROIの完全一致を検出cropの完全一致とは主張しない。
- 通常529件、ONNX対象451件が成功。Pool方向指定の互換/再利用、実CLI opt-out、曖昧な2領域の維持、PBTを含む。fmt/diff/対象Clippy成功（既存の本体17警告＋example 1警告）。前版12d3378の全5CIも成功。
- 検出枠の先頭/末尾欠落、重複で復旧を控える例、36pxのbands2例の短いままの認識、URL/空白残差は未解決。次は36pxの候補が採用されない原因を元RGB/準備済みcrop/各chunkの停止理由へ分ける。未知実スキャンの汎化/高速化は未確認。

## 2026-10-19: 孤立した下側の小画線を本文と一緒に認識

- 先行仕様 `docs/dense_crop_probe.md` / `docs/isolated_trailing_mark.md`。固定79画像を全走査し、条件に合う36画像88cropを詳細記録。6470 chunk推論で36px2例の読点だけが幅25pxへ孤立すると分離。上下余白の各案は79crop中12/13/9/11件が悪化するため棄却。
- 前/後の文脈結合は30/100×2字体×左右の8条件で120字完全一致。幾何・PBTの未実装3 Redと実モデルRedを先に確認。単一下側8連結の小画線だけ、近い字間と本文の支持・実入力容量・列群上限を満たすと前へ戻す。RGB・bbox・隣接空白metadataは保持し、語句置換は加えない。
- 診断結果閲覧前に新24対照を固定し、元79と全103×4経路×3正規化の1236文字距離が非悪化。CLI 10本文組が改善、他402組はbyte同一。36px2例はbandsで117→0、120→0/各120字。新太字和文も135→4/135字。
- PageRecognizer30/100は各103本文・全metadataが同一。CLI DEIM設定の改善3例は生DEIMの有効cropが0で帯fallbackによる認識と確認した。runtimeに同じfallbackがない差を次に検証する。
- 新太字対照のCLI両設定は、単語誤りが各1→2（none/widthの計4指標）へ退行。文字距離の採用ゲートと分離して全件記録。通常533件＋追加境界2件、ONNX対象459件、fmt/diff/対象Clippy成功（既存の本体17警告＋example 1警告）。前版e9b1bd3の全5CIも成功。
- 79×2 runtime比較元の本文は全byte再現。本文/metadata等の全1185ファイルをSHA検証してzipへまとめ、PRのコード差分を確認できる状態を維持した。途中の未完了runtimeによる採点失敗は、同じ推論が完了してから採点だけを再実行。未知資料への汎化/高速化は未確認。

## 2026-10-19: DEIM未検出のPageRecognizerに確認済みの帯を追加

- 先行仕様 `docs/page_band_fallback.md`。モデル実行前に一般短文/反復/2行24画像を固定し、元103と全127画像を比較。画素支持・Horizontal・最低0.9・native超過帯の安全分割確認を小さいRed→Greenで実装した。供給された同じpoolのみ使い、DEIMで検出済みの経路とpublic Pool Autoは維持する。
- 無条件案は1524文字指標中60退行、画素支持/Horizontal案は2286中24退行。最低confidence案は文字距離非悪化でもI反復から0.906の英文を誤追加したため不採用。3案の全出力・採点・sourceを保存し、全幅とchunkの生出力を分離してnative容量超過帯の採用条件を追加した。
- 最終案はCLI DEIM/bands、PageRecognizer30/100の評価/既定設定、全6経路×none/width/strictの2286画像別文字距離が非悪化。生文字距離104本文組が改善、654組はbyte同一、4組は文字距離が同じ。4組の数字→横画線誤読は改善に数えない。既存非空runtime出力の全本文/bbox/confidenceは同一。
- 未検出の120文字本文と疎な3語を復旧し、URIは両モデル・両optionsで全文一致。I反復の英文誤追加は抑えた。CLI両経路の太い縦棒の偽文字54→0も確認した。
- 単語指標は6本文組のnone/width、計12指標が1→2へ悪化する。丸字体和文2画像×30モデルの2設定と太字和文1画像×100モデルの2設定。全件の単語総誤りは減るが、全指標非悪化とは主張しない。
- 通常553件、ONNX対象489件が成功。方向・native容量・画素保存/PBT・load/from_pools・並列順・options・エラー再利用を検査。ONNX無効時にエラーを空ページ成功へ変える退行も先行Redから修正した。fmt/diff/対象Clippy成功（既存の本体17警告＋example 1警告）。前版d716fe4の5CIも成功。
- 反復数/文字種、短い和文のモデル差、句読点/URL/空白、検出枠欠落と曖昧な重複は残る。既知資料・合成の開発回帰で、未知資料への汎化や高速化は未確認。次は反復文字の生出力とsanitizerの削除を分離する。

## 2026-10-19: 画像が支持する反復数の保持と同一行の包含断片整理

- 先行仕様 `docs/repeated_glyph_diagnosis.md` / `docs/visible_repetitions.md` / `docs/contained_line_context.md`。160画像をOCR前に固定し、raw正読204入力が旧sanitizerで削られることを分離した。独立した同baselineの画線群が文字数を支持する場合だけ反復上限を外す。文字種/語句の推測置換はしない。
- stateless/Session/Poolの画像付き単行・ページ断片へ適用。text-only sanitizerと生API・括弧方針・confidenceは維持する。原画像を変更せず追加推論もない。
- 初回案の9文字指標退行を不採用として保存。全体と包含断片のDEIM重複を実モデルRedで再現し、複数の独立支持・span/gap・native比率・認識confidence・文字量が揃う全体だけを残した。別行の結合や検出scoreとの混同はない。
- 前段11b6118比でページ287×6×3と単行263×3×3の全7533文字指標・全7533単語指標は非悪化。2511本文組は392改善・2006byte同一・113同値変更。既知フォントの合成対照を独立文書とは数えず、文字種の同値誤りを改善へ数えない。
- 通常568件・ONNX対象513件・example18件・Python2件が成功。API保存出力への純粋選択再生は実モデル44ページで、単行の保持結果は最終CLI33件で一致。最終CLIページ574プロセスは新規実行した。
- stdout終端LFを本文へ混ぜた初回誤採点も保存し、元truthを変えず前後同じ厳密1LF抽出で再採点。全stdout/本文のroundtripとSHAを検査。全27表、途中案、実行/再生の区別とsource/binary/image SHAは `docs/benchmarks/visible-repetitions/` に収録。
- 次はI/l/1/O/0等の文字種誤読、複数成分を持つ文字やcrop不足を切り分ける。縦向きの画素支持や未知実資料の汎化は未解決。Clippy成功（既存17+画素走査スタイル1、example既存1警告）、高速化は主張しない。

## 2026-10-19: 全文枠のない行の全体文脈を診断

- 先行仕様 `docs/missing_line_context_probe.md`。固定184画像×30/100モデル×2設定の736本文組と1368候補を調べ、全文前景を含む1232候補だけを全文採点。製品の変更はない。
- 一律置換は150文字指標退行、native内＋confidence>=.9でも54、native内＋既存平均以上でも24の退行があるため、そのまま採用しない。全文枠がある既存の正読をconfidenceだけで入れ替える反例を保存した。
- 全文枠のない混合A1 JPEGの既定APIでは、原画像候補で30モデル13→4誤り、100モデル9→4誤り。100のconfidence=.893は固定.9では除外するため、floorのない相対層を仕様追記後に全件比較した。
- 全文被覆/native内/帯1本/非空既存出力/未被覆画線/既存平均以上の層では86候補・258文字/単語比較が非悪化、生38組改善。原画像/帯を独立に採点し、真値で選択しない。全287画像とCLIは未検証で、本番の精度改善とは称さない。
- 未被覆和集合・閾値・不正RGB・全角/strict・部分crop非採点・相対層を6テスト（80固定seedの性質を含む）でRed→Green。全342cropの元RGB一致を同じRust decoderで確認し、前段archiveを別rootへ展開した全診断JSONもbyte一致した。`docs/benchmarks/missing-line-context-probe/` にsource/全ログ/SHAを保存。
- 前段bae38d3の全5CIが成功。次はこの欠落回復候補を全287画像・API4設定とCLIで実測し、未被覆のJPEG微小画線と余白の影響を切り分ける。

## 2026-10-19: 未被覆画線を持つ単一横書き行の全体文脈確認

- 先行仕様 `docs/missing_line_context_recovery.md`。元bbox和集合の外に画線があり横書き帯が1本の場合だけ、同じprimaryのnative内・全文被覆の原画像/帯をHorizontalで確認する。元結果の非空白文字数加重平均を厳密に上回る候補のみ採用し、文字置換・文書固有条件・追加モデルはない。
- 純粋方針6件（40例の拡大/移動PBTを含む）と実モデルRed→Green。混合A1 JPEGの既定APIは30モデル13→4誤り、100モデル9→4へ改善し、固定.9で除外していた.893候補も扱える。元の評価設定/空結果/optionは維持する。
- 270f85a比、全287×6×3＋単行263×3×3の7533文字/単語指標が非悪化。2511本文組は51生距離改善・2450byte同一・10同値変更。改善は既存合成対照24、新反復対照27で、既存実資料20画像は同一。未知実資料holdoutとは扱わない。
- API全1148ページとCLI全574プロセスを新規実行。APIでは122ページで166追加呼出を記録し、元RGB/既存結果と記録応答を製品方針へ戻す全1148件のpolicy再生も一致。最終CLIの変更した全本文と対照41件、単行30件の新規byte一致を検査し、単行789予測は全byte再利用した。
- 通常574件・ONNX対象521件・example18件、fmt/diff/対象Clippyが成功（既存本体18+example1、新規警告なし）。証拠は `docs/benchmarks/missing-line-context/`。元API推論binary SHAの未取得を明示し、検証binaryとdriver/source/model/input/output・全候補応答の根拠を区別する。
- 追加のCLI det=.3は構造補正false/trueの前後4実行を検査し、15→4 / 8→4文字誤り、各正規化の単語指標も非悪化。
- 次は単行画像で物理的に存在する閉じ括弧の欠落、複数成分の反復文字、文字種、復旧bboxの余白を診断する。追加推論の統制速度/未知資料汎化は未確認。

## 2026-10-19: 印字された閉じ括弧を単行APIでも画線支持で保持

- 先行仕様 `docs/literal_closers_probe.md` / `docs/literal_closers.md`。5括弧種・4配置・単独/連続と対照の320画像をOCR前に固定し、既存263と全583画像のrawを30/50/100で捕捉。正しかったraw184組が単行の未対応括弧削除で崩れていた。
- 一律保持は73文字指標・108単語指標で退行。捕捉後に探索した .5/.7/.9/.99 の確率案も退行するため棄却し、全候補本文・step確率・停止理由・正式評価表を保存する。高確率の幅違い/挿入空白を正解と扱わない。
- 7画線を6字へ落とす純粋Redと凍結比較元CLIの `ABC)` → `ABC` の実モデルRedを先行。元fragment候補が長く、空白なし・実効横書き・有効RGB・既存の独立成分数支持を満たす場合だけ保持する。追加推論なし。text-only/raw/候補APIとconfidence・RGB、ページfragmentと全caller sourceは維持する。
- d6a67fe比、全583×3モデル×3正規化=5247の文字・単語指標が非悪化。1749本文組は104完全一致改善・1622byte同一・23生距離同値変更。既存263の全789本文は同一。raw正読の未回復80組は共通高さ44・成分数36の条件で残る。
- before新960 CLIを実行し、旧789は前段SHAと今回raw由来の全byteを確認して再利用。after全1749 CLIを新規実行し、raw/RGBによる製品再生と全一致。公開stateless/Session/Pool、batch/単一Session batch/cascade、raw/候補、エラー後の再利用、RGB/confidenceも実モデルで検査した。
- 通常576件・ONNX対象526件・example18件、fmt/diff/対象Clippy成功（既存本体18+example1、新規警告なし）。ONNX件数には明示環境変数がないとno-opの診断2件を含む。既存幾何PBT48例も成功した。
- `docs/benchmarks/literal-closers/` に凍結binary/source/driver/model/input SHA、全本文・stdout・失敗案・9組の実CLI表と9組の診断表を保存。別rootへ全SHA検証展開した実CLI9 CSV/MD＋監査のbyte一致と、モデル不要の1749製品/fragment再生も確認した。元forwardとportable検証のbinaryを区別する。
- PRのファイル数を抑えるため旧dense-lineの939 memberをSHA付きZIPへ固定。別rootの全24 CSV/MD＋監査一致を確認してから重複before/truthだけを外し、後続scriptが読むafterは維持した。前段d6a67feの全5CI成功も確認済み。
- 次は残る複雑な成分・背の低い括弧、幅/空白・文字種誤読を診断する。全ページの今回の新規再測定、未知実資料の汎化、速度改善は主張しない。目的全体は継続する。

## 2026-10-19: 閉じ括弧と本文の高さを分ける診断

- 先行仕様 `docs/closer_geometry_probe.md`。丸ゴシック/明朝・20/40px・PNG/JPEG75の640文字画像と図形16画像をOCR前に固定し、656 fixtureの通常テストをRed→Green。既存583と全1239画像を30/50/100で診断した。
- test-onlyで成分の最小高さを使う案と、本文の共通高さ3/4を維持しcloserを個別に確認する案を、純粋Red→Green・閉じた境界/負例・拡大/移動/位置PBT64例で先行。本文の整列を弱める反例を独立に検査する。製品の関数・callerは変更しない。
- 既存1749 rawは前段archive/source/model/RGBと現在baselineの全CLI byteを確認して再利用。新1968 forwardからraw・上位2確率・停止理由を捕捉し、元RGBのPNG保存/再decodeの全画素一致と全3717の既存成分支持との一致を検査した。
- 全1239×3×3=11151の文字/単語指標で両高さ案は非悪化、raw正読64組を回復できる。元の未回復80組中44と新対照20を含む。本文整列案は64完全一致改善・3626byte同一・27生距離同値変更で、単純緩和の誤字種の同値変更1組を追加しない。次の製品/API検証対象にする。
- 空白なし一律保持も6文字・32単語指標で退行し、一律fragment保持は213文字・338単語指標で退行する。`情）報` にraw `報告)報`、`情報）` にraw `情報){` 等の反例を全件保存した。真値や確率を選択に使わない。
- 通常583件、ONNX lib263件、純粋6+保存raw/RGB再生1件が成功。Clippy testsの新規成分走査警告を直し、前後全3717の枠/全方針を再生一致。対象Clippy成功、新規ファイル警告0（本体既存18、lib test既存28/うち18重複などをログに残す）。fmt/diffも成功。
- `docs/benchmarks/closer-geometry-probe/` に全6方針本文・9 CSV/MD・原raw/RGB・成分・失敗案・元forward/portable別binary/source/driver/spec/入力SHAを保存。別rootでモデル/原画像/probeなしに全表+監査をbyte再現し、凍結binaryの3717成分/選択再生も一致した。前段02ad642の全5CIも成功。
- これは診断で、64組を製品で回復済みとは主張しない。元rawの正読を壊す329組中265組（成分数216、x区間重複47、eligible除外2）が残り、元80組の残り36も含む。次は本文整列案の製品適用/API・CLI検証と、複数成分/接触・空白の診断を進める。未知資料の汎化/速度は未確認で、目的全体は継続する。

## 2026-10-19: 低い閉じ括弧を本文整列の支持で単行APIへ回復

- 先行仕様 `docs/body_closer_recovery.md`。純粋Redと公開APIの `資料」` → `資料` の実モデルRedを先行し、本文の共通高さ3/4・closer高さ1/2と交差3/4、字数/x区間/幅・空白/方向/RGBの支持を満たす場合に元fragmentを保持する。追加推論や文書語句の置換はない。
- 既存の成分抽出を共用するが、ページfragment/反復の厳密な共通高さ条件は維持する。全3813の従来支持/fragmentと保存値が一致し、新body条件も別の凍結referenceと全件一致。parseq/cached/page/app caller sourceはbyte同一で、既存実ページ回帰も成功した。
- 呼出sourceから25/45字のCascade境界を確認し、OCR前に32画像を追加固定してfixture Red→Green。全1271×3単行と32×3開始bucketの3909本文組を、before/afterとも実APIで新規実行した。新境界96 rawだけ追加捕捉し、単行confidenceは全3813で元rawと前後bit一致した。
- 9f6a5e2比、全11727文字/単語指標は非悪化、64完全一致改善・3818byte同一・27生距離同値変更。改善は旧1239の64組（元の80残差中44と前段新対照20）で、境界は同一。全変更91と境界/不採用反例/系統対照を含む257 CLIを前後とも新規実行し、実APIと全本文/厳密1LF抽出が一致した。
- native字数からのモデル再構成と実観測を区別し、隔離sourceでpool結果受取にtest-only観測を追加。getter未実装の0/96 Redから前後各96履歴を実観測し、全本文/confidenceが元の実APIとも一致した。履歴は19[30]・13[30,50]・32[50]・32[100]で変更0。観測hookを製品へ含めない。
- 実境界で新しく復旧したcloserが再認識先を変えるケースの精度まで確認したとは主張しない。純粋画線23/24/43/44字では+1復旧を別テストした。境界のraw正読24組も未回復で、全体の353組中64回復・289未回復（成分数240、x重複47、eligible除外2）を記録する。
- 通常は全586+境界1の587件、ONNXは全539+境界1+再生1の541件、example18件が成功。実モデルscalar/Pool/batch/単一Session batch/cascade・raw/候補・confidence/RGB・エラー再利用と、幾何/位置/拡大/明度PBT64例を検査。fmt/diff/対象Clippy成功、新規警告0（既存本体18、lib test28/うち18重複などは保存）。前段9f6a5e2の全5CIも成功。
- `docs/benchmarks/body-closer-recovery/` に実行前凍結binary/source/driver、原raw/画像・全API/CLI・実dispatch source/patch・全18 CSV/MD・失敗/再生ログを保存。別rootで同梱採点toolによる全表+監査のbyte一致と、3813製品/reference/従来支持/fragment/confidenceおよび境界成分の再生一致を確認した。元forwardとportable検証・隔離計測を区別する。
- ページ287×6の今回の全再実行、未知実資料の汎化、速度は主張しない。次は接触/複数成分と空白・文字種、特に低いcloserに隣接する明示的な単語空白とモデルの空白省略を追加対照で診断する。目的全体は継続する。

## 2026-10-19: 末尾閉じ括弧の前の単語空白を画線から回復

- 先行仕様 `docs/closer_word_spaces_probe.md` / `docs/trailing_closer_word_space.md`。固定したABCの80対照で、fc22d35が旧厳密支持版より7本文組・14単語指標を悪化させることを発見した。文字距離の改善だけでは空白省略の退行を防げない。40joined画像は旧対照とRGB/rawが同じで、独立新規材料に数えない。
- 純粋Redと実モデル公開Sessionの `ABC』` → `ABC 』` のRedを先行。末尾closer1個・本文3字以上・従来/本文の成分支持、本文内2個以上の小gapに対する明確な末尾gapが揃った場合にだけASCII空白1個を支持する。薄罫線/極端な高さ/遠すぎる画線・複数closer・語中空白などは元方針へ戻す。追加推論・語句やcloser種の置換はない。
- 候補評価前にDEF・2font・20/40px・PNG/JPEGと細/高い罫線の176対照を追加固定しfixture Red→Green。全1527×3×3=13743の文字/単語指標がfc22d35比・旧厳密支持比とも非悪化、発見した14単語退行も解消した。4581本文組は56生距離改善・うち38完全一致、4525byte同一。旧1271の3813本文は全て同一。未知実資料holdoutとは扱わない。
- 旧3813 rawを親SHA/全本文照合で再利用し、新80と176はraw/processed各768 forwardを新規実行。afterは全4581を公開Sessionで再推論し、全本文が候補とbyte一致、confidenceがrawとbit一致した。変更全件と新176対照の和集合190画像×3モデルの570 CLIを前後とも新規実行し、全APIと厳密1LF抽出の本文が一致。最初の80にも旧厳密支持/比較元CLI各240を保存した。
- 閉じたgap/幅/高さ境界・RGB/方向・PBT64例と、3モデルのstateless/Session/Pool/batch/単一Session batch/fragment・Cascade開始3/2/1・エラー再利用を検査。純粋22/23/24/42/43/44字は空白込みの字数を確認する。最初の境界テストは長い同字反復自体が短縮される誤ったfixtureだったため、異なる文字の巡回へ直して検査した。実画像で新たに25/45字の再認識先が変わる精度まで確認したとは主張しない。
- `docs/benchmarks/trailing-closer-space/` に原画像/decoded RGB・全本文/真値・確率/停止理由・API/CLI・正式18表・失敗/成功ログ・source/binary/input/model SHAを保存する。全API/CLIのlint前sourceと、等価な `?` へ整えた最終sourceのモデル不要4581選択/fragment再生を区別した。直接pathの予備API実行は中断し、copy済みbinaryで全件やり直した。
- 元の未回復289に加え、新対照を含むraw正読の未回復は330組。次はprefix空白、複数成分/接触、文字種・復旧bboxの余白を診断する。ページ全件の新規再測定・速度・未知資料汎化は未確認で、目的全体は継続する。fc22d35の全5CIは成功した。
- 通常595件＋追加境界1件、ONNX CI対象493件と明示rootの再生1件、example18件が成功。fmt/diff/対象Clippy成功、新規警告0（既存本体18・lib test28/うち18重複）。最初のignored全実行はmodel/charset環境変数不足で停止し、CIと同じ明示設定で再検査した。
- 別rootでarchiveと全memberのSHAを検証し、同梱toolの18表＋監査と、最終製品sourceの4581選択/fragment再生が一致した。環境変数を明示した既存実資料1件も成功。

## 2026-10-19: 狭いadvanceで先頭閉じ括弧の後ろの単語空白を回復

- 先行仕様 `docs/leading_closer_word_space.md`。前段の新しい未回復41組が全て先頭closer＋空白だった。KLM・24/36px・2font・PNG/JPEGと左罫線176対照をOCR前に固定しfixture Red→Green。末尾のgap条件を先頭へ対称化するtest-only案は185本文変更・108文字/262単語指標退行で棄却した。`）ABC` →raw `) ABC` の全角字幅の余白を空白と扱う反例を全件保存する。
- 第2候補の評価前にPQR・18/32pxと左罫線の176対照をさらに固定。closerの画線幅＋先頭gapが本文最大高さ以内に収まる追加支持を仕様化した。閾値探索や語句/closer種類の置換はせず、不確かな場合はae42fceを維持する。純粋Redと公開Sessionの `) ABC` → ` ABC` のRedを先行して製品Greenにした。追加推論なし。
- 全1879×3×3=16911文字/単語指標がae42fce比で非悪化。5637本文組は78生文字/単語距離改善・うち77完全一致、5559byte同一。65はraw正読の後処理損失の回復、12はモデル省略空白の補完で完全一致、1は `)POR` → `) POR` の空白だけ改善しO/Qは維持した。変更はABC12・DEF24・KLM18・PQR24、元1271の3813本文は全て同一。
- 元1527の4581 rawを親archive SHAと全before/fragment/実API照合で再利用し、KLM/PQRはraw/processed各1056 forwardを新規実行。after公開Sessionを全5637入力で新規実行し、全本文が候補とbyte一致、confidenceがrawとbit一致した。全変更＋新PQR176の和集合194画像×3モデルの582 CLIを前後とも新規実行し、全APIと厳密1LF抽出で本文が一致した。
- gap/幅/高さ/advanceの閉じた境界・全角字幅反例・語中/複数/二重/非ASCII空白・RGB/方向・PBT64例、3モデルのstateless/Session/Pool/batch/単一Session batch/fragment/raw/候補・エラー再利用・Cascade開始3/2/1を検査。純粋22/23/24/42/43/44字は空白込みの字数を確認するが、今回の実画像が25/45字の再認識先を変える精度まで検証したとは主張しない。
- 通常603件、ONNX CI対象503件と明示rootの5637再生1件、example18件が成功。fmt/diff/対象Clippy成功、新規警告0（既存本体18・lib test28/18重複）。ae42fceの全5CIも成功。前段のONNX集計は最後の5件完了前の488だったため、既存archiveの完了ログSHAを検証し493へ文書を訂正した。archiveと評価表/SHAは変更せずverificationへ根拠を追記した。
- `docs/benchmarks/leading-closer-space/` に原画像/decoded RGB・raw/成分/確率/停止理由・全方針本文JSON・実API/CLI・36表・失敗/成功監査・source/binary/input/model SHAを保存。方針別TXTは同梱scriptで再生し重複を省く。別rootで全SHAと成功/失敗案36表＋両監査のbyte一致、同じ製品executableで全5637選択/fragmentの再生一致を確認する。
- 最初の41組中35回復・全角側の6未回復。新対照込みではraw正読の損失400→335、元1271の289も残る。追加画像は既知fontの合成開発対照で、未知実資料holdoutではない。次は全角closer側の空白と複数成分/接触、文字種の誤読を切り分ける。全ページ新規再測定・速度・未知資料汎化は未確認で、目的全体は継続する。

## 2026-10-19: 文字内部の分離成分と閉じ括弧・空白の支持

- 先行仕様 `docs/contained_closer_components.md`。未回復335組は成分過多177・不足92・x重複47・空白/advance19。`資料)` の料は大きいbbox内に分離画線があり、3字を4成分と数えていた。元RGB/8近傍/area閾値は維持し、他bboxの4辺すべての厳密な内側の成分だけをcloser用viewで除く。ページfragmentと反復の従来count gateは元成分を維持する。
- 候補出力を見る前に、料金/回路/ijI/2048の240字形、回路図の空白80、全体枠32の352対照を固定しfixture Red→Green。包含だけの案は419本文変更・238完全一致改善だが、空白を省いたrawの括弧だけ戻して64単語指標が退行し棄却した。
- 第2案評価前に結果表/図面/回A図/XYZ・24pxの320対照を固定。Unicode幅2のpitch支持を試したが、包含のない既存正読も再解釈して16文字/108単語指標退行で棄却した。第3案前に経路図/図画/回B図/UVW・40pxの320対照をさらに固定し、元fixture/truthを変更していない。全追加992は既知fontの合成開発対照で未知実資料holdoutとは扱わない。
- 採用案は包含でviewが実際に変わる場合だけ適用する。幅2本文が2字以上ある全角/混合では、幅2の最大bbox幅＋本文最大gapをセル幅とし、字高・pitch・先頭advanceを確認して1個のASCII空白を支持する。全角1セル付近を狭い空白と扱わず、細い実括弧にはこのセル支持内だけ字高/5の幅を要求する。新runtime依存は固定 `unicode-width 0.2.2` の表示幅のみ。語句/closer種類を置換せず追加forwardなし。
- 全2871×3×3=25839文字/単語指標がed453b9比で非悪化。8613本文組は727byte変更・7886byte同一。664はいずれかの指標/正規化で改善、63は全指標同値で改善に数えない。生文字距離では524改善・203同値変更。470完全一致改善のうち359はraw正読の後処理損失回復、111はモデル省略ASCII空白だけを補って完全一致。既存1879の未回復335は299へ、36組回復した。
- 元5637 rawを親archive全SHAとbefore/fragment/実API照合で再利用し、新992画像はraw/processed各2976 forwardを新規捕捉。after全8613を実Sessionで新規実行し、全本文が候補とbyte一致、confidenceがrawとbit一致。元bbox/area・従来count gate・fragmentの全8613再生も一致。全変更＋新320の和集合570画像×3モデルの1710 CLIを前後とも新規実行し、厳密1LF抽出後の本文が全APIと一致した。
- 純粋画線と公開Sessionの `資料)` → `資料` のRedを先行し製品Greenにした。4辺/同bbox/入れ子/部分重複/dotとgap/advance/セル幅の境界、4種類のPBT各64例、3モデルのstateless/Session/Pool/batch/単一Session batch/raw/候補/fragment・RGB/confidence・エラー再利用・Cascade開始3/2/1を検査。通常616件、ONNX CI対象519件と明示rootの8613再生1件、example18件が成功。fmt/diff/対象Clippy成功、新規警告0（既存本体18・lib test28/18重複）。ed453b9の全5CIも成功した。
- 保存native字数では25/45字の新しい閾値跨ぎ0・字数減少0。これは実dispatch履歴と区別する。今回の復元で再認識先が変わる実画像の精度、全ページの新規再測定・速度・未知資料汎化は未確認。以前の未検出行復旧で記録した単語12指標退行は別baselineで、最新実装での再評価は次段とする。
- `docs/benchmarks/contained-closer-components/` に3段階の原画像/decoded RGB・真値・全方針本文/成分/確率/停止理由JSON・実API/CLI・54表/3監査・全ログ・実行前凍結source/binary/Cargo/input/model SHAを保存する。画像/診断/本文を3archiveに分け、方針別TXTは同梱scriptで生成して重複を省く。別rootで全member SHAと54表＋3監査のbyte一致を確認した。
- 新対照込みのraw正読損失は930→571。包含でまとまらない部品・接触・字種・空白は残る。次は以前記録したページ評価の単語退行12指標を最新実装で再評価し、原因を確認する。単行の未回復も継続する。目的全体は継続する。
- 別rootで3archiveの全23819 member SHA、54表＋3監査のbyte一致と、同じ製品executableの8613件の原bbox/area・従来支持・fragment・製品選択の再生一致を確認した。

## 2026-10-19: 過去の単語距離退行の再監査と短い語尾の文脈確認

- 先行仕様 `docs/page_word_regression_reaudit.md`。最新基準の全1,722本文組が直近ページ評価と同一で、過去の単語距離退行12指標が残っていることを確認。
- 生モデル内の空白と、分割で孤立した短い語尾を分離。未実装Redと公開APIの誤読Red→Green。長い帯の安全分割成功後だけ、原RGBの2区間を同モデルで確認し、容量・文字数・前半本文一致・confidenceの制限を満たす候補を採用する。
- 287画像×6経路×3正規化、5,166文字/単語指標が全件非悪化。太字1画像の100モデル2設定で、生文字距離4→1、単語距離2→1。他1,720本文組はbyte同一。過去の単語指標退行は12→8。完全一致の改善とは数えない。
- 通常619件・ONNX CI対象523件・example18件、fmt/diff/Clippy成功（新規警告0）。生API/単行処理の経路は維持し、APIの行数/矩形も全組不変。primary50のページ行列・未知実資料・速度は未評価。
- 丸字体の2区間×9余白×3モデルを追加診断。30では余分な空白を消せても、100で余分な空白とconfidence上昇が同時に起きる反例があり、余白の自動選択は採用しない。次は残る生空白の画素支持とprimary50のページ対照を検討する。

## 2026-10-19: primary50ページ監査と生空白の余白合意案を棄却

- 先行仕様 `docs/page_primary50_audit.md`。公開APIの任意manifestを記録する診断exampleを、空/重複/不正IDのRedと起動Red→Greenで追加。1a66fce/4ba9fd6のsource・target・実binaryを分離し、認識前に全SHAを固定した。
- 30/100の40対照で前段の本文/bbox/confidenceを全再現。primary50の287画像×2設定は前後とも新規実行し、全574本文組・metadataが同一、全1,722文字/単語指標は非悪化。前段6経路の5,166比較と合わせて8経路6,888比較が非悪化。今回の精度改善とは数えず、前段6経路を新規推論したとも表現しない。
- `docs/raw_space_margin_consensus_probe.md` を先行し、固定146→66→90入力で3モデル・4余白条件を診断。合意のみのAは第1段84文字/84単語指標で退行。画像高に対する白列幅を加えたBは当初48完全一致回復・退行0でも、紙余白対照で16本文組・32文字/32単語指標を壊した。両案を棄却し、製品処理は変更しない。
- 白紙追加で画像高14→18に対し前景高/白列幅12は不変、Bのgateだけfalse→trueとなる反例を保存。第2段の33改善も実空白の欠落が残り、完全一致に数えない。次は紙余白に依存しない画線高の支持を、別仕様と不変性テストで検討する。
- 新example2テスト・ONNX全example20件、fmt/diff/Clippy成功（新規警告0、既存lib18）。4ba9fd6のCI5ジョブ成功。全302入力/1,208区間×3モデル・両案の確率/選択/棄却例とsource/binary/input SHAを `docs/benchmarks/primary50-and-spaces/` に保存する。

## 2026-10-20: 句読点の誤空白を画線高・advanceと実モデル合意で確認

- 先行仕様は `docs/ink_height_space_support.md` から `docs/punctuation_space_advance_integration.md`。
  紙を除いた前景高だけのCは実空白/点ノイズで退行し棄却。
  面積で絞った成分高と句読点境界のDも横65%の実空白9組で18文字/18単語指標を壊し、棄却した。
- 成分幅のEと、水平被覆幅/文字数のFを、仕様とPBTを先行して共通の新1,152入力で比較した。
  全3,446入力×3モデル×3正規化では両案退行0。Fの114文字距離改善/70完全一致回復がEの82/56を上回り、Fを製品へ接続した。
  既存のraw/候補デコーダは保持し、同Sessionで最大3回だけ純白余白候補を確認する。語句置換は追加しない。
- AS初回exportの30モデル14 ROIは分割されていたため採点せず、診断用 `--whole-roi` をRed→Greenで追加して全件再測定した。
  元から単一区間の6,898記録は候補/確率まで一致した。初回・再測定とも保存し、不都合な入力を落とさない。
- 保存実NN結果の製品全件replayは62,028文字/単語指標に退行0。
  影響する全100入力ID＋反例/負例の158対照は、基準9fc/Fの8単行経路で新規認識し、
  全7,584本文/f32 confidenceがreplayと一致、11,376文字/単語指標に退行0。単行CLI14プロセスも3回復・4負例保持を確認した。
- ページ287×8経路はFで全件新規実行、6,888文字/単語指標に退行0。
  丸字体24px半縮小の30モデル2設定で全文を完全一致へ回復し、生/width文字距離1→0・単語距離2→0。
  他2,294本文組、全API行数/矩形と他confidenceは不変。比較前値はsource/入力/SHA照合済み9fc参照を再利用した。
- 通常639件、実モデル556件、example23件、fmt/diff/Clippy成功、新規警告0。
  先行9fcのCI5ジョブも成功。全段階の原画素/真値/候補・棄却例・凍結source/binary/model/input SHAと再現手順は
  `docs/benchmarks/punctuation-space-context/` に保存する。新規NN測定と製品replayを区別する。
- 固定fontの開発対照で、未知資料/速度は未検証。丸字体36pxの誤空白などは残る。
  次は `docs/punctuation_space_proportional_controls.md` に先行固定した比例送りの実空白対照を検査し、目的全体を継続する。

## 2026-10-20: 比例組版でF/Gを棄却し、自動空白削除を撤去

- 比例送りの先行仕様に従い、CoreTextの同じ実font file・glyph・空白advanceと `palt=0/1` を確認して新576入力を固定した。
  Fは累積4,022入力で11本文組・22文字/22単語指標を壊した。Rust選択の両本文方針でも退行を再現し、前段の採用判断を撤回した。
- Gは元のASCII空白を除いた本文平均を全3候補が厳密に上回る条件を、解析/追加認識前に別仕様へ固定した。
  既存4,022入力の12,066元結果を再取得し、全本文/f32平均が一致。新576入力を含む全18,978 forwardも正常終了した。
  累積4,598入力では旧6・新26本文組、64文字/64単語指標が退行したためGも棄却し、Rust製品へ未実装とした。
- 先行撤去仕様 `docs/punctuation_space_rejection.md` と実モデルの4 Red→5 Greenを経て、FのSession/Pool/batch/断片/standaloneへの接続を削除した。
  `parseq.rs` は9fcとbyte同一、`cached.rs` もtest-only import以外はbyte同一。Fのselector/画像支持はcfg(test)だけに限定した。
- F/撤去後/9fc基準の実CLI計21プロセスで、比例空白3退行を解消し、旧Fの3回復も撤回した。撤去後の全7本文は9fcとbyte同一。
  通常639件・ONNXスイート556件・example23件、fmt/Clippy成功、新規警告0。前段3b47552のCI5ジョブも全成功した。
- F/Gの全画像・真値・候補/確率・成功/失敗・元NN再取得のbit一致・凍結source/model/binary/input SHAと撤去のRed/Greenは
  `docs/benchmarks/punctuation-space-context/` に保存する。異なる母集団の退行数や、NN forward数とRust選択replayの指標数を混同しない。
- この段階の最終製品は9fcと同じ動作で、新しい精度改善とは主張しない。ページの誤空白回復も撤回され、以前の未解決箇所は残る。
  confidenceと余白合意だけで空白を削る方法を既定経路へ追加せず、他の画線/検出/分割の改善を継続する。目的全体は未完了。

## 2026-10-20: 1画素で接触した本文2字と閉じ括弧の支持

- 先行仕様 `docs/single_pixel_body_bridge.md`。モデルが正読した `情報)` を単行処理が `情報` にしていた。
  本文2字が1画素で接触し、2文字を1成分と数えていたため、支持用maskだけでその1画素を分ける。
  元RGB/collector/NNを保ち、空白なし3字・端closer1個/表示幅2本文2字の、既存fallbackが失敗した場合だけを扱う。
- 純粋分離・字形支持・実画像と公開SessionのRedを先行し、最後のfallback追加後にGreenにした。
  幅/高さ/gap、2接続・内部点・矩形mark・不正RGB/方向、紙/移動/灰度のPBT64例を検査した。
- 既存2,871・空白4,598画像の保存raw比較は全件非悪化。新本文/3font file/18・30px/palt/実空白/JPEG/紙と
  非文字ノイズを追加認識前に3,648画像へ固定し、全10,944 forwardが正常終了した。
  Auto回転128画像も除外せず3モデルのtensor bit一致を先に検証し、元Horizontal指定384 forwardも別監査した。
  実入力RGB全3,776件のSHAを照合した。
- 各群のnone/width/strictで文字・単語指標に退行0。生の完全一致を旧25・新6の計31本文で回復し、
  生距離同点の17本文は半角closerをそのまま保持してwidthだけ改善した。未知資料の改善率とは扱わない。
  旧raw正読後の損失は571→546で、分離した字形部品などの課題は残る。
- 凍結した実製品処理の33,735組は事前診断と全件一致。全選択/新ノイズ/方向指定の2,745実API組は
  raw/本文/f32 confidenceが固定値と一致し、全選択/ノイズ2,361実CLIも一致した。単行CLIは終端LFだけ1個除いた。
  ページ287×8経路は新規実行し、6,888文字/単語指標・全本文/行数/矩形/confidenceが不変だった。
  比較前値はsource/input SHA照合済み9fc保存値を再利用した。
- 通常648件、ONNXスイート570件、example23件が成功、新規Clippy警告0。
  証拠とモデル不要の再現手順は `docs/benchmarks/single-pixel-body-bridge/`。目的全体は継続する。

## 2026-10-20: 分離した本文部品と端closerの支持

- 先行仕様 `docs/detached_body_components.md`。情の大部品が外側成分と下端だけを共有し、正読 `情報)` が後処理で `情報` になる損失を、支持用の部品groupで解消した。
  3辺が厳密内側/1辺だけ一致、白列・所属・幅/高さ/gapを検査し、空白なし3字の最後の単行fallbackだけを扱う。元RGB/NN/fragment/raw/confidenceを保持する。
- 小さな純粋テスト、実画像、公開SessionのRed→Greenと64例PBTを実施した。稲穂/樹枝10,944入力は適用0だったため、条件を変えず情人/情理10,944入力の仕様を別途先行した。
  64 Auto回転も保持し、元Horizontal192 forwardと全3形状tensor bit検査を追加した。新規65,856 forwardは全件正常終了。
- 全99,591組のnone/width/strict文字/単語指標は非悪化。本文変更394組のうち生完全一致308、width完全一致384組を回復し、10組は全指標同点だった。旧raw正読後の損失546→520。
  既知の情部品を用いた適用対照であり、未知資料の改善率とは扱わない。前段1画素案の別基準監査28回復とも合算しない。
- 製品全件replayが凍結方針と一致、実公開API1,882組のraw/本文/mean bitと実単行CLI682プロセスも全件一致した。今回は新規ページNNなし。fragment経路と関係関数のbyte一致を確認し、前段287×8の実測とは区別する。
- 通常655件・ONNX CI対象582件・example23件、新規Clippy警告0。証拠とモデル不要の再現は `docs/benchmarks/detached-body-components/`。
  内側closer、Auto方向の紙依存、別の接触字形、Latin高さの課題が残り、目的全体を継続する。

## 2026-10-20: 保存Auto方向比較と語中closerの支持

- 横組み192画像×3モデルの既存rawをAuto/元Horizontalで対にした。生完全一致388回復だが、strictでは100モデルの文字距離が2組悪化したため、一律Horizontal変更は採用しない。
  新規NNは0、正式36表と192原RGB/真値を独立再現した。`docs/benchmarks/auto-orientation-saved/` に反例も保存する。
- 次は `docs/internal_detached_closer.md` の中央mark/左右部品の固定支持案をtest-onlyで先行した。純粋Red→Green、6テスト/PBT64と旧99,591組で文字/単語退行0、旧9組を生完全一致へ回復した。
  新対照5,088画像と元Horizontal56画像の15,432全ROI forwardは全件正常終了し、全群/全正規化の文字/単語指標が非悪化だった。
- 最後の単行fallbackへ接続し、純粋/実SessionのRed→Greenを確認した。本文変更136組のうち、生完全一致89組・width完全一致121組を回復し、5組は全指標同点。旧raw正読後損失520→511。
  分離部品の既知字形を用いた適用対照であり、未知資料の改善率とは扱わない。
- 製品115,023組の保存raw再生が凍結案と一致。実API2,176組のraw/本文/mean bitと実CLI712件も全件一致し、通常661件・ONNX593件・example23件、新規Clippy警告0。
  今回の新規ページNNは0、fragment経路のsource不変性を確認した。証拠とモデル不要再現は `docs/benchmarks/internal-detached-closer/`。
- 負例生成で白列がなく停止した入力も保持し、NN前に中央cutの合成方法を仕様へ追記した。支持条件は追加結果に合わせず、全元画素/非重複を384 pre-JPEG組で独立再構成した。
  combined接触部品やLatin高さ、真縦組みを含むAuto方向の新対照などが残るため、目的全体を継続する。

## 2026-10-20: 接触主成分と分離した本文部品を同時に支持する

- 先行仕様 `docs/owned_parts_body_bridge.md`。正読 `情報」` の本文主成分が1画素で接触し、情の大部品も分離していたため、既存2案ではcloserを支持できなかった。主成分の回収と支持maskだけの同じ1画素cut/中心距離/全画素所属/左右groupで救済する。文書別置換や元RGB/NNの変更はない。
- 純粋Red→Green/PBT64を先行した。CoreText10,944+16画像とFreeType10画像は固定案の幾何到達性0でNNを実行せず、全対照を保存した。原seedのPillow/FreeType BASICをRGB同一で再現し、palt/RAQMを捏造せず既知の情報/報情と非文字対照7,632画像を認識前に固定した。
- 22原画像は旧RGBと重複し、7,610画像は未重複。全3形状の元Auto/有効Horizontal tensor bit照合、実1,116描画/576 pre-JPEG合成/72目視の後、全22,896 raw forwardと元Horizontal24×3を正常に測定した。既知字形の条件検査であり、未知資料のholdoutとは扱わない。
- 旧新137,991組の全正規化に文字/単語退行0。変更82組のうち生完全一致51組・width完全一致70組、8組は全指標同点だった。生回復は旧14、新37（未重複RGB30/既存RGB7）。条件/modelの組を独立標本と数えない。旧raw正読後損失511→497、新noise1,152画像の変更0。
- 全3モデルの公開Session Redを先行し、最後の単行fallback接続後にGreenにした。未使用import整理後も条件を保ち、最終sourceの全製品再生・API2,602組・CLI946件が一致、通常671件/ONNX608件/example23件、新規Clippy警告0。新規ページNNは0でfragment関係source不変性を検査した。
- 証拠とモデル不要の独立再現は `docs/benchmarks/owned-parts-body-bridge/`。狭い丸括弧・Latin・複数closer、真縦組みを含むAuto方向の対照などが残るため、目的全体を継続する。
- 独立復元の初回は容量不足で終了し、検証済みの前段復元コピーだけを整理して同じpacketから再実行した。正式306表と3監査、元画素/実入力/描画/API/CLI/meanが一致し、復元後の実Rust137,991組と7,632×3形状tensor bit照合も成功した。失敗も含む終了記録と全SHAを保存した。

## 2026-10-21: FVB全件比較の継続と追加対照の認識前固定

- 凍結FVBの原runはPage30/50を完了したが、Page100/CLIが180分で中断した。
  完了分は各921入力・3,684 Page結果・11,064公式CSV行を独立照合し、各35入力/経路組がraw改善、悪化0。
  同じ画像の別モデル/設定を独立改善へ合算しない。中断分はpartialとして保持し、gate成功に含めない。
  `0c937c4`の分割retryはPage100/CLIを両sourceで再測定中。全44,256 CSV行と全経路の採否は未確定。
- [追加実資料の事前仕様](real_vertical_additional_controls.md)に沿い、既存2冊以外の
  一葉全集/地獄変の6画像・11採点表記を認識前に固定した。
  [原画像・GT・取得記録](benchmarks/real-vertical-additional-controls/)を保存し、OCR/model loadは0。
  傾き/退色で確定できなかった候補も残す。FVBの途中の比較へ入力を混ぜない。
- [余白と容量guardの診断](vertical_padding_capacity_diagnostic.md)は保存Q/Sの432組だけを使い、追加NN0。
  guard緩和を採用する証拠は得られず、本体へ変更していない。
- [CLI縦列fallback案FVC](vertical_cli_fallback_proposal.md)の共通helperだけを隔離sourceでRed→Greenにした。
  通常lib447成功/39ignored、padding/移動のPBT128例が通過。
  新72対照を固定し、432 geometry条件/282cropを独立画素照合、文字範囲の欠け0を確認した。
  [証拠一式](benchmarks/vertical-cli-fallback/)を保存した。CLI接続・実モデル認識はまだ行っていない。
  FVB全件gateの整理後に、既知実CLI Red/Greenと新規比較へ進む。本体推論は引き続きO。

### FVC隔離CLI adapterの継続（追加モデル認識0）

- [adapter仕様](vertical_cli_fallback_adapter.md)を先行し、prepared crop/bucketの2 Redと
  結果再利用の1 Redを経て、関連8テストをGreenにした。隔離CLIへ接続後、通常lib451成功/40ignored、
  既存app/引数/feature無効のintegration21件が成功し、ONNX CLI/testをno-runでfreshにコンパイルした。
- feature無効の実CLIは本文/白×DEIM off/on×asset missing/dummy×前後の16 processで、
  exit/stdout/stderr/bodyが一致した。最初の8 process比較ではmissing-fileよりfeature-errorを先と
  仮定したtest期待が失敗したため、製品を変えず前提を分け、失敗と出力を保存した。
- [追加packet](benchmarks/vertical-cli-fallback/adapter-before-nn-v1.json)へ両source・Red/Green・build・
  実CLI出力を保存した。モデル・native実行物は配布しない。隔離したONNX CLIは前後別SHAで固定し、
  実モデルRed/Green・新72精度比較は未実行。本体推論は引き続きO、FVB全件比較は継続中。
- 先の`21f3626`は通常/実モデルを含むCI全5ジョブ成功。これは本体OのCIであり、隔離FVCのモデル検証ではない。

## 2026-10-21: Linux Page全3モデル完了とモデル選択差の診断

- FVBの100モデル4 shardが完了。921入力・3,684 Page結果・11,064公式CSV行を独立照合し、
  生8入力/経路組改善、全正規化の入力別文字/語悪化0。30/50各35組と独立画像の改善数へ合算しない。
  [完了Page packet](benchmarks/vertical-page-fallback/completed-pages-linux-v1.json)に全6 Page jobを保存した。
  [再検証仕様](vertical_completed_page_evidence.md)・実packet integration Red→Greenにより、
  モデルなしで計11,052保存Page結果・全33,192 CSV行・悪化0を再検証。CLI4 shardは進行中。
- [保存モデル選択差の仕様](vertical_page_model_choice_diagnostic.md)→分類4 Red→6 Green、
  既存reader含む29件Green。7,368比較から99入力ID/99RGB・424観測を分類し、
  モデルなし/実model SHA照合の両replayでcases全文が一致した。14prefix観測は既知合成3画像に重複する。
  H/Oや実CLI内部traceではなく、新policy・閾値変更・追加NNは0。
- 中断した旧CLI runの成功1 commandでは、71字入力が24字prefixとして出力されていた。
  source/fresh build/argv/framingは照合したが、親jobの最終stampがないため診断専用とする。
  完了artifactやgateへ含めず、後続の実CLI cascade traceを候補へ残す。
- `76d4b37`はCI全5ジョブ成功。本体OのCIでありFVCのNN検証ではない。

## 2026-10-21: FVB全8経路gate成功、本体統合

- [完全run39532673209](https://github.com/bokuweb/ndlocr-lite-rs/actions/runs/39532673209)の
  10完了artifact/4種類、全44,256CSV行・22,128比較行を手元でも検算しremoteと完全一致した。
  入力別文字/語の悪化0、新FVB本文raw7経路組改善、CLI3,684 processの本文変更0。
  CLI4 raw jobs・全CSV・全比較・独立検算を[完全gate packet](benchmarks/vertical-page-fallback/complete-cli-and-gate-linux-v1.json)へ保存した。
- [統合前仕様](vertical_page_fallback_integration.md)に従い、現在の278 bound filesがO baselineと同じことを確認後、
  FVB推論5ファイルと13 testsを移した。除去はcfg(test) captureだけで、残りはrustfmt後の凍結sourceと一致する。
  Root fmt/default lib439成功・38ignored、default Clippyは既存13警告。
- 最初のlocal test buildは容量不足で実行前に停止し、同じsourceをstrip noneで再buildして成功した。
  [統合proof](benchmarks/vertical-page-fallback/root-integration-proof-v1.json)へ失敗/成功・source対応を保存した。
  本体CIへ実vertical_fallback_page回帰とONNX Clippyを追加した。全Root CIの結果は追って確認する。
- 次はこのFVB本体を基準に追加実資料6画像/11表記を測る。FVC新72と、Oの既知CLIモデル選択traceは独立gateを維持する。
