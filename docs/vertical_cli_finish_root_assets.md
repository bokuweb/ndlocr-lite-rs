# Rootのknown finish回帰を固定assetへ結ぶ

Root v1のfresh CLI回帰2件はGreenだったが、quality回帰はNN前に
`charset not found: ndlocr/src/config/NDLmoji.yaml` でRedになった。
旧private実験の配置へ依存するCLI既定charsetをテストから除く。
本体のCLI既定値や推論処理は変更しない。

固定fixtureに加え、Cargo manifest rootからmodels内の3既定モデルとNDLmoji.yamlを
絶対パスへ解決し、全fileの存在を必須assertする。Cli::try_parse_fromへ4パスを明示し、
モデル不足をskipしない。固定画像、3 quality設定、全文/f32 bits一致、実際にqualityが
初期値を変えるassertは維持する。修正はtest-onlyで、新fresh libtestで再検証する。
初回失敗とbefore/after source/fresh build/実行記録を保持する。
