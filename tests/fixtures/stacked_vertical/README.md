# 二段の縦書き本文（開発回帰）

Source: National Diet Library, *国立国会図書館月報* 683号 (2018.3), page 20,
「次世代室の謎に迫れ！」, via [ndl-lab/ndlocr-lite](https://github.com/ndl-lab/ndlocr-lite/blob/d001dad2013ceabb5736ff8d637474e443eb73c2/resource/digidepo_11048278_po_geppo1803_00021.jpg).
Upstream licence: [CC BY 4.0](https://github.com/ndl-lab/ndlocr-lite/blob/d001dad2013ceabb5736ff8d637474e443eb73c2/LICENCE).

`body.png` is the unscaled RGB crop `[126,133,724,758]` from the source JPEG.
Its pixels and manually transcribed truth were frozen before model inference.
`manifest.json` records the original hash, crop, acceptance and limitations.
`frozen-before-inference.json` records pre-inference file hashes.

The truth includes all 47 physical columns (top 23, bottom 24) in reading order.
Paragraph indentation and the author-name spacing are layout, not transcribed spaces.
Fullwidth Latin/digits are the fixed transcription choice; scoring does not normalize them.
This development body region excludes frames, pictures, caption and heading. It is not a full-page
or independent holdout evaluation; training overlap is unknown.
