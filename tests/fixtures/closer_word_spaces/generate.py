"""Freeze explicit word spaces and their paired controls before OCR."""
from pathlib import Path
import hashlib,io,json,unicodedata,zipfile
import PIL
from PIL import Image,ImageDraw,ImageFont
r=Path(__file__).resolve().parent;sha=lambda b:hashlib.sha256(b).hexdigest();fonts=[];rows=[]
for key,name in [('gothic','ヒラギノ角ゴシック W3.ttc'),('mincho','ヒラギノ明朝 ProN.ttc')]:fonts.append((key,next(p for p in Path('/System/Library/Fonts').iterdir()if unicodedata.normalize('NFC',p.name)==name)))
with zipfile.ZipFile(r/'corpus.zip','w',compression=zipfile.ZIP_DEFLATED,compresslevel=9)as z:
 def put(name,data):
  item=zipfile.ZipInfo(name,date_time=(1980,1,1,0,0,0));item.compress_type=zipfile.ZIP_DEFLATED;z.writestr(item,data)
 for font_id,font_path in fonts:
  font=ImageFont.truetype(str(font_path),28)
  for family,close in [('round',')'),('full_round','）'),('corner','」'),('double_corner','』'),('lenticular','】')]:
   for prefix in [False,True]:
    for spaced in [False,True]:
     gap=' 'if spaced else'';text=close+gap+'ABC'if prefix else'ABC'+gap+close;x0,y0,x1,y1=font.getbbox(text);im=Image.new('RGB',(x1-x0+16,y1-y0+16),'white');ImageDraw.Draw(im).text((8-x0,8-y0),text,font=font,fill='black')
     for fmt in ['png','jpeg75']:
      id=f'{family}_{"prefix"if prefix else"suffix"}_{"space"if spaced else"joined"}_{font_id}_{fmt}';image='images/'+id+('.png'if fmt=='png'else'.jpg');truth='truth/'+id+'.txt';buf=io.BytesIO()
      if fmt=='png':im.save(buf,format='PNG')
      else:im.save(buf,format='JPEG',quality=75,subsampling=2)
      data,gt=buf.getvalue(),text.encode();put(image,data);put(truth,gt);rows.append({'id':id,'closer':close,'prefix':prefix,'spaced':spaced,'font':font_id,'format':fmt,'truth':text,'width':im.width,'height':im.height,'image_member':image,'truth_member':truth,'image_sha256':sha(data),'truth_sha256':sha(gt)})
m={'font_size':28,'margin':8,'pillow_version':PIL.__version__,'generator_sha256':sha(Path(__file__).read_bytes()),'fonts':[{'id':key,'file':p.name,'sha256':sha(p.read_bytes())}for key,p in fonts],'archive_sha256':sha((r/'corpus.zip').read_bytes()),'rows':rows};(r/'manifest.json').write_text(json.dumps(m,ensure_ascii=False,indent=2)+'\n',encoding='utf-8');print('frozen',len(rows),'images',m['archive_sha256'])
