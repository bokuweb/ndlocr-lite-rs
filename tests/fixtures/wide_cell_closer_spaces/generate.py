"""Freeze new wide, mixed, and narrow spacing controls before the second candidate."""
from pathlib import Path
import json,hashlib,io,unicodedata,zipfile
import PIL
from PIL import Image,ImageDraw,ImageFont
r=Path(__file__).resolve().parent;sha=lambda b:hashlib.sha256(b).hexdigest();fonts=[];rows=[]
for key,name in [('gothic','ヒラギノ角ゴシック W3.ttc'),('mincho','ヒラギノ明朝 ProN.ttc')]:fonts.append((key,next(p for p in Path('/System/Library/Fonts').iterdir()if unicodedata.normalize('NFC',p.name)==name)))
with zipfile.ZipFile(r/'corpus.zip','w',compression=zipfile.ZIP_DEFLATED,compresslevel=9)as z:
 def put(n,b):
  i=zipfile.ZipInfo(n,date_time=(1980,1,1,0,0,0));i.compress_type=zipfile.ZIP_DEFLATED;z.writestr(i,b)
 for font_id,path in fonts:
  font=ImageFont.truetype(str(path),24)
  for body_id,body in [('results','結果表'),('drawing','図面'),('mixed','回A図'),('latin','XYZ')]:
   for family,c in [('round',')'),('full_round','）'),('corner','」'),('double_corner','』'),('lenticular','】')]:
    for prefix in [False,True]:
     for spaced in [False,True]:
      gap=' 'if spaced else'';text=c+gap+body if prefix else body+gap+c;x0,y0,x1,y1=font.getbbox(text);h=y1-y0+16;im=Image.new('RGB',(max(x1-x0+16,h*2),h),'white');ImageDraw.Draw(im).text((8-x0,8-y0),text,font=font,fill='black')
      for fmt in ['png','jpeg75']:
       id=f'{body_id}_{family}_{"prefix"if prefix else"suffix"}_{"space"if spaced else"joined"}_{font_id}_{fmt}';name='images/'+id+('.png'if fmt=='png'else'.jpg');gt='truth/'+id+'.txt';buf=io.BytesIO();im.save(buf,format='PNG'if fmt=='png'else'JPEG',**({}if fmt=='png'else{'quality':75,'subsampling':2}));b=buf.getvalue();t=text.encode();put(name,b);put(gt,t);rows.append({'id':id,'body':body,'closer':c,'prefix':prefix,'spaced':spaced,'font':font_id,'size':24,'format':fmt,'truth':text,'width':im.width,'height':im.height,'image_member':name,'truth_member':gt,'image_sha256':sha(b),'truth_sha256':sha(t)})
assert len(rows)==320
m={'font_size':24,'pillow_version':PIL.__version__,'generator_sha256':sha(Path(__file__).read_bytes()),'fonts':[{'id':key,'file':p.name,'sha256':sha(p.read_bytes())}for key,p in fonts],'archive_sha256':sha((r/'corpus.zip').read_bytes()),'rows':rows};(r/'manifest.json').write_text(json.dumps(m,ensure_ascii=False,indent=2)+'\n');print('frozen',len(rows),'images',m['archive_sha256'])
