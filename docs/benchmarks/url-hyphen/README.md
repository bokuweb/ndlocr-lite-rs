# URL/パスの末尾ハイフン

先行仕様 `../../url_literal_hyphens.md`。
モデルや画素を変えず、URL/パスを英文のハイフネーションとして結合しない。
通常の英文結合は維持し、資料名/単語を使った置換は加えていない。

前版5a8dad9と候補の固定binaryで、全20画像×DEIM/bands×2版=80プロセスを実行。
入力/モデル/真値/binary SHAはmanifest、実行argv/終了状態はruns、
前後本文・真値・全6指標CSV/Markdownを保存した。auditに前版全40本文の再現も記録。

全120件の画像別文字指標（none/width/strict）が非悪化。
神戸市の原画像/JPEG90/JPEG75は両検出器、半分PNGはDEIMだけ変化し、
それぞれURL2本の行末ハイフンと物理改行を保持する。他33本文はbyte同一。
原画像の既定DEIMは生74→70、width26→22、strict58→56 /1034字。
bandsは生78→74、width29→25、strict60→58。

`unstructured-control.json` と本文では構造処理を無効にした前後がbyte同一。
今回の改善をモデル/切り出しの改善としては扱わない。
ドメイン内hyphenやURL内underscore、文字の誤読は別の残差として残る。
モデルの追加推論・画像加工・ネットワークアクセスは発生しない。

元資料5件とその機械変形なので、未知資料/実スキャンの汎化や速度改善は未確認。
