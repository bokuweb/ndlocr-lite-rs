# 拡張scopeの最終採点件数を固定値から分離する（全v2 NN前）

新wrapperの追加監査で、再利用runnerのcompare末尾に旧scopeの74,700固定assertが残っていることを確認した。
v2は79,020行なので、そのままでは認識完了後の件数確認に失敗する。run39591981411を中止し、状態/失敗も保持する。
実際のworker開始/NN数はrun状態を確認して記録し、推測で0とはしない。

旧run39568828241は完了しており、旧event SHA/保存harnessは不変として保持する。
現在の共通runnerだけで、最終採点件数の判定をplan由来のexpectedとactualの完全一致へ一般化する。
入力/GT/推論Rust/モデル/認識関数/CSV計算/悪化判定は変えない。差分前sourceを保存する。
74,700/79,020の両方を受け、欠落/余剰/空計画を拒否する契約を未実装Red→Greenにし、
旧11契約と新bundle/union契約を再実行する。修正は最終件数検証だけであることもsource bridgeで確認する。
新event SHAでbuildと全56 workerを測定し、旧/中止runをv2の成功へ流用しない。
