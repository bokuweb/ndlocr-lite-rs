from pathlib import Path
import PIL,hashlib,json,unicodedata
from PIL import Image,ImageFont,ImageDraw
root=Path('tests/fixtures/long_line_split');(root/'images').mkdir(parents=True,exist_ok=True);(root/'truth').mkdir(exist_ok=True)
texts={
'library':'The library preserves historical books, maps, photographs, and newspapers so that future readers can study their contents.',
'observations':'Each observation includes the station name, measurement time, instrument settings, and a short note describing the local conditions.',
'japanese':'図書館では資料の保存と公開を継続し、利用者が必要な情報を探せるように目録を整備しています。新しい電子資料を追加するときは、文字の読み上げ、画面の拡大、色の変更、検索結果の表示などを確認し、担当者が検査の記録と改善の内容をまとめて公開します。',
'uri':'https://example.invalid/library/collections/historical-maps/metadata/observation_records/measurement_conditions/technical-report_2026.html',
'control_en':'Readers can search the catalogue online.',
'control_ja':'図書館の資料を検索して閲覧できます。',
}
fontdir=Path('/System/Library/Fonts');find=lambda n:next(p for p in fontdir.iterdir() if unicodedata.normalize('NFC',p.name)==n)
fonts=[('gothic',find('ヒラギノ角ゴシック W3.ttc')),('mincho',find('ヒラギノ明朝 ProN.ttc'))]
rows=[]
for font_name,font_path in fonts:
 for size in [24,36]:
  font=ImageFont.truetype(str(font_path),size)
  for key,text in texts.items():
   x0,y0,x1,y1=font.getbbox(text);underlined=key=='uri';height=y1-y0+8+(3 if underlined else 0);width=x1-x0+8
   im=Image.new('RGB',(width,height),'white');draw=ImageDraw.Draw(im);draw.text((4-x0,4-y0),text,font=font,fill='black')
   if underlined:draw.line((4,height-4,width-5,height-4),fill='black',width=1)
   stem=f'{key}_{font_name}{size}';image=root/'images'/(stem+'.png');im.save(image);(root/'truth'/(stem+'.txt')).write_text(text)
   rows.append({'id':stem,'image':str(image),'truth':text,'underlined':underlined,'font':font_name,'font_size':size,'sha256':hashlib.sha256(image.read_bytes()).hexdigest()})
meta=json.loads(Path('tests/fixtures/residual_spaces/paragraph.json').read_text())
for label,name in [('paragraph_raw','paragraph.png'),('paragraph_normalized','paragraph-normalized.png')]:
 source=Path('tests/fixtures/residual_spaces')/name;image=root/'images'/(label+'.png');image.write_bytes(source.read_bytes());text=meta['truth'];(root/'truth'/(label+'.txt')).write_text(text)
 rows.append({'id':label,'image':str(image),'truth':text,'source':str(source),'source_metadata':'tests/fixtures/residual_spaces/paragraph.json','sha256':hashlib.sha256(image.read_bytes()).hexdigest()})
manifest={'pillow_version':PIL.__version__,'fonts':[{'name':n,'path':p.name,'sha256':hashlib.sha256(p.read_bytes()).hexdigest()} for n,p in fonts],'rows':rows}
(root/'manifest.json').write_text(json.dumps(manifest,ensure_ascii=False,indent=2)+'\n');print('frozen',len(rows),'images')
