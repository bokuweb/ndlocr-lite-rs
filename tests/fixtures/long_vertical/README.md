# 長い縦書き列の分割回帰

`manifest.json` は6列の原文、描画矩形、PNGのSHA256を保持する。
通常31/34/36字の3列を初回推論前に固定し、すべて全文一致した。
続いて中心16字目を三・二・一とした31字の3列を別の推論前に固定した。
変更前 `2d8270a` は三で1編集、二で2編集、一は全文一致だった。
初回固定時刻と追加固定時刻はmanifestに記録する。
これらは合成した開発回帰であり、独立holdoutではない。

Pillow 12.3.0で、Arial Unicode（48px）の各文字を52px送りで描く。
フォントSHA256はmanifestに保存する。フォントそのものは配布しない。
同じフォントを持つ環境では次の描画手順で再現できる。

```python
from PIL import Image, ImageDraw, ImageFont
# truth はmanifest内の各原文。font_pathは同じhashのローカルフォント。
font = ImageFont.truetype(font_path, 48)
image = Image.new("RGB", (68, 52 * len(truth) + 16), "white")
draw = ImageDraw.Draw(image)
boxes = []
for i, char in enumerate(truth):
    position = (10, 8 + i * 52)
    draw.text(position, char, font=font, fill="black", anchor="lt")
    boxes.append(draw.textbbox(position, char, font=font, anchor="lt"))
image.save(output_path)
```

原文・字形矩形は採点専用で、分割器と推論には画像とモデル寸法だけを渡す。
モデルを使わない回帰は字形内部を切らないこと、実モデル回帰は6列194字の全文一致を要求する。
短い既存縦列の4/83編集、横書き回帰も維持する。
実資料、別フォント、詰め組み、数字だけの列への汎化は未確認。
