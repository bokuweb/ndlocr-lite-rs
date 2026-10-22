from pathlib import Path
import PIL,hashlib,json,unicodedata
from PIL import Image,ImageDraw,ImageFont
r=Path('tests/fixtures/page_band_fallback');sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest()
fontdir=Path('/System/Library/Fonts');fonts=[]
for key,name in [('gothic','ヒラギノ角ゴシック W3.ttc'),('mincho','ヒラギノ明朝 ProN.ttc')]:
 p=next(p for p in fontdir.iterdir()if unicodedata.normalize('NFC',p.name)==name);fonts.append((key,p))
texts={'single_ja':'雨','sentence_ja':'資料を確認します。','sentence_en':'Read and verify.','repeated_i':'IIIIII','repeated_digits':'111111','two_rows':'作業の記録を残します。\n次の担当者が確認します。'};rows=[]
for fname,fpath in fonts:
 font=ImageFont.truetype(str(fpath),24)
 for kind,text in texts.items():
  lines=text.split('\n');boxes=[font.getbbox(line)for line in lines];width=max(b[2]-b[0]for b in boxes)+12;step=36;height=(len(lines)-1)*step+max(b[3]-b[1]for b in boxes)+12
  image=Image.new('RGB',(width,height),'white');draw=ImageDraw.Draw(image)
  for i,(line,(x0,y0,_,_))in enumerate(zip(lines,boxes)):draw.text((6-x0,6+i*step-y0),line,font=font,fill='black')
  for fmt in ['png','jpeg75']:
   stem=f'{kind}_{fname}_{fmt}';path=r/'images'/(stem+('.jpg'if fmt=='jpeg75'else'.png'));truth=r/'truth'/(stem+'.txt')
   if fmt=='jpeg75':image.save(path,quality=75,subsampling=2)
   else:image.save(path)
   truth.write_text(text,encoding='utf-8');rows.append({'id':stem,'image':str(path),'truth':text,'image_sha256':sha(path),'truth_sha256':sha(truth),'font':fname,'font_size':24,'format':fmt})
(r/'manifest.json').write_text(json.dumps({'pillow_version':PIL.__version__,'fonts':[{'name':n,'file':p.name,'sha256':sha(p)}for n,p in fonts],'rows':rows},ensure_ascii=False,indent=2)+'\n',encoding='utf-8');print('frozen',len(rows))
