from pathlib import Path
import PIL,hashlib,json,unicodedata,random
from PIL import Image,ImageDraw,ImageFont
root=Path('tests/fixtures/dense_line');(root/'images').mkdir(exist_ok=True);(root/'truth').mkdir(exist_ok=True)
old=json.loads(Path('tests/fixtures/long_line_split/manifest.json').read_text());old_text=next(x['truth'] for x in old['rows'] if x['id']=='japanese_gothic24')
texts={'library':old_text,'observations':'観測記録には測定場所、時刻、使用した機器、設定の変更、周辺の環境などを記載します。計画に沿って資料を集めた後は、担当者が数値の単位と日付を照合し、欠けている項目や重複した記録がないかを確認します。整理した結果は一覧表にまとめ、利用方法と注意点を添えて公開します。'}
fontdir=Path('/System/Library/Fonts');find=lambda n:next(p for p in fontdir.iterdir() if unicodedata.normalize('NFC',p.name)==n)
fonts=[('maru',find('ヒラギノ丸ゴ ProN W4.ttc')),('bold',find('ヒラギノ角ゴシック W6.ttc'))];rows=[]
def save(stem,im,text,**extra):
 path=root/'images'/(stem+('.jpg' if extra.get('transform')=='jpeg75' else '.png'))
 if path.suffix=='.jpg':im.save(path,quality=75,subsampling=2)
 else:im.save(path)
 (root/'truth'/(stem+'.txt')).write_text(text)
 rows.append({'id':stem,'image':str(path),'truth':text,'sha256':hashlib.sha256(path.read_bytes()).hexdigest(),**extra})
for fname,fpath in fonts:
 for size in [24,36]:
  font=ImageFont.truetype(str(fpath),size)
  for key,text in texts.items():
   x0,y0,x1,y1=font.getbbox(text);im=Image.new('RGB',(x1-x0+8,y1-y0+8),'white');ImageDraw.Draw(im).text((4-x0,4-y0),text,font=font,fill='black')
   for transform in ['original','jpeg75','half']:
    variant=im.resize((im.width//2,im.height//2),Image.Resampling.BILINEAR) if transform=='half' else im
    save(f'{key}_{fname}{size}_{transform}',variant,text,font=fname,font_size=size,transform=transform)
# All graphical/spacing controls are fixed before any new inference.
for name in ['blank','rule','solid','speckles','horizontal_marks','vertical_bars','spaced_short']:
 im=Image.new('RGB',(3000,48),'white');draw=ImageDraw.Draw(im);truth=''
 if name=='rule':draw.line((0,35,2999,35),fill='black',width=1)
 elif name=='solid':draw.rectangle((0,0,2999,47),fill='black')
 elif name=='speckles':
  rng=random.Random(20261019)
  for _ in range(180):draw.point((rng.randrange(3000),rng.randrange(48)),fill='black')
 elif name=='horizontal_marks':
  for x in range(0,3000,20):draw.rectangle((x,22,x+11,23),fill='black')
 elif name=='vertical_bars':
  for x in range(0,3000,30):draw.rectangle((x,5,x+14,42),fill='black')
 elif name=='spaced_short':
  font=ImageFont.truetype(str(fonts[0][1]),36)
  for x,char in zip([4,1450,2940],'図書館'):draw.text((x,0),char,font=font,fill='black')
  truth='図 書 館'
 save(name,im,truth,kind='spacing_control' if truth else 'graphic_control')
font=ImageFont.truetype(str(fonts[1][1]),36)
for name,text in [('repeated_i','I'*120),('repeated_digits','0123456789'*12)]:
 x0,y0,x1,y1=font.getbbox(text);im=Image.new('RGB',(x1-x0+8,y1-y0+8),'white');ImageDraw.Draw(im).text((4-x0,4-y0),text,font=font,fill='black');save(name,im,text,kind='repetition_control')
(root/'manifest.json').write_text(json.dumps({'pillow_version':PIL.__version__,'fonts':[{'name':n,'path':p.name,'sha256':hashlib.sha256(p.read_bytes()).hexdigest()} for n,p in fonts],'rows':rows},ensure_ascii=False,indent=2)+'\n');print('frozen',len(rows),'new controls')
