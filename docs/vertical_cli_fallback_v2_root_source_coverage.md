# Rootのsource固定を実ファイルから作る

接続後packetの独立readerが、旧Root baselineのfile集合を継いだfreezeに、新規登録した
quality testとfixture3点が含まれないことを検出した。v1/v2の実テスト成功は保存するが、
完全なsource固定の成功として扱わない。Root runtime src92の同値監査とは別の問題である。

次のfreezeはsrc/tests/examples/benchesの全Rust、tests/fixturesの全file、Cargo.toml/lockを
実ファイルから列挙する。Rustのliteral include_bytes!/include_str!/include!の参照先も
加え、既存のCLI parserが埋め込むoptions-plan ZIPを固定する。参照先の欠損を無視しない。
未対応の動的includeやRoot外の参照があれば、勝手に省かず失敗させる。

全fileのSHAをビルド前後・認識前後で一致させ、新fresh lib/CLI/testを作る。
全新helper/fixtureを含む完全な集合で再度CLI2・quality1・通常lib・feature無効を確認する。
同じruntime/モデル/GTの全件NN gateを重複実行する必要はない。
初回reader assertionと不完全な旧freezeは履歴として保持し、後の結果へ付け替えない。
