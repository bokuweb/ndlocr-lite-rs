from pathlib import Path
import PIL,hashlib,json,unicodedata
from PIL import Image,ImageFont,ImageDraw
root=Path('tests/fixtures/short_line_order');(root/'images').mkdir(parents=True,exist_ok=True)
labels=['で上昇','上昇で','進行','行進','第１章','第一章','本日','日本','国会','会国','AB','BA','ABC','CBA','12','21','123','321','AAB','ABA','ＢＡ','ＡＢ','一二三','三二一','あいう','ういあ','カキク','クキカ','により','よりに','令和','和令']
fontdir=Path('/System/Library/Fonts')
font=lambda name:next(p for p in fontdir.iterdir() if unicodedata.normalize('NFC',p.name)==name)
fonts=[('gothic24',font('ヒラギノ角ゴシック W3.ttc'),24),('mincho24',font('ヒラギノ明朝 ProN.ttc'),24),('gothic16',font('ヒラギノ角ゴシック W3.ttc'),16)]
rows=[]
for variant,path,size in fonts:
 f=ImageFont.truetype(str(path),size)
 for index,text in enumerate(labels):
  x0,y0,x1,y1=f.getbbox(text); im=Image.new('RGB',(x1-x0+2,y1-y0+2),'white');ImageDraw.Draw(im).text((1-x0,1-y0),text,font=f,fill='black')
  image=root/'images'/f'{variant}_{index:02d}.png';im.save(image)
  rows.append({'id':image.stem,'image':str(image),'truth':text,'sha256':hashlib.sha256(image.read_bytes()).hexdigest()})
(root/'manifest.json').write_text(json.dumps({'pillow_version':PIL.__version__,'fonts':[{'variant':v,'font':p.name,'size':s,'font_sha256':hashlib.sha256(p.read_bytes()).hexdigest()} for v,p,s in fonts],'rows':rows},ensure_ascii=False,indent=2)+'\n')
