"""Freeze literal closers and controls before model inspection; fonts stay local."""
from pathlib import Path
import hashlib, io, json, unicodedata, zipfile
import PIL
from PIL import Image, ImageDraw, ImageFont

root = Path(__file__).resolve().parent
sha = lambda data: hashlib.sha256(data).hexdigest()
fonts = []
for key, name in [('gothic', 'ヒラギノ角ゴシック W3.ttc'), ('mincho', 'ヒラギノ明朝 ProN.ttc')]:
    path = next(p for p in Path('/System/Library/Fonts').iterdir() if unicodedata.normalize('NFC', p.name) == name)
    fonts.append((key, path))
cases = []
for family, opening, closing in [('round', '(', ')'), ('full_round', '（', '）'), ('corner', '「', '」'), ('double_corner', '『', '』'), ('lenticular', '【', '】')]:
    for body_id, body in [('latin', 'ABC'), ('digits', '123456'), ('japanese', '資料')]:
        for position, text in [('prefix', closing + body), ('suffix', body + closing), ('internal', body[:1] + closing + body[1:]), ('balanced', opening + body + closing)]:
            cases.append((f'{family}_{body_id}_{position}', text))
    cases.append((family + '_alone', closing))
    cases.append((family + '_double', 'ABC' + closing * 2))
for key, text in [('plain_latin', 'ABC'), ('plain_digits', '123456'), ('plain_japanese', '資料'), ('square_suffix', 'ABC]'), ('square_prefix', ']ABC'), ('curly_suffix', 'ABC}'), ('curly_prefix', '}ABC'), ('period', 'ABC.'), ('japanese_period', '資料。'), ('question', 'ABC?')]:
    cases.append(('control_' + key, text))
assert len(cases) == len(dict(cases)) == 80
rows = []
with zipfile.ZipFile(root / 'corpus.zip', 'w', compression=zipfile.ZIP_DEFLATED, compresslevel=9) as archive:
    def put(name, data):
        info = zipfile.ZipInfo(name, date_time=(1980, 1, 1, 0, 0, 0))
        info.compress_type = zipfile.ZIP_DEFLATED
        archive.writestr(info, data)
    for font_id, font_path in fonts:
        font = ImageFont.truetype(str(font_path), 28)
        for case, text in cases:
            x0, y0, x1, y1 = font.getbbox(text)
            image = Image.new('RGB', (x1 - x0 + 16, y1 - y0 + 16), 'white')
            ImageDraw.Draw(image).text((8 - x0, 8 - y0), text, font=font, fill='black')
            for fmt in ['png', 'jpeg75']:
                id = f'{case}_{font_id}_{fmt}'
                image_member = 'images/' + id + ('.png' if fmt == 'png' else '.jpg')
                truth_member = 'truth/' + id + '.txt'
                buffer = io.BytesIO()
                if fmt == 'png': image.save(buffer, format='PNG')
                else: image.save(buffer, format='JPEG', quality=75, subsampling=2)
                data, truth = buffer.getvalue(), text.encode('utf-8')
                put(image_member, data); put(truth_member, truth)
                rows.append({'id': id, 'case': case, 'font': font_id, 'format': fmt, 'width': image.width, 'height': image.height, 'truth': text, 'image_member': image_member, 'truth_member': truth_member, 'image_sha256': sha(data), 'truth_sha256': sha(truth)})
manifest = {'font_size': 28, 'margin': 8, 'pillow_version': PIL.__version__, 'generator_sha256': sha(Path(__file__).read_bytes()), 'fonts': [{'id': key, 'file': path.name, 'sha256': sha(path.read_bytes())} for key, path in fonts], 'archive_sha256': sha((root / 'corpus.zip').read_bytes()), 'rows': rows}
(root / 'manifest.json').write_text(json.dumps(manifest, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')
print('frozen', len(rows), 'images; archive', manifest['archive_sha256'])
