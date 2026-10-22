"""Freeze generic containment and enclosure controls before candidate evaluation."""
from pathlib import Path
import json,hashlib,io,unicodedata,zipfile
import PIL
from PIL import Image,ImageDraw,ImageFont
r=Path(__file__).resolve().parent;sha=lambda b:hashlib.sha256(b).hexdigest();fonts=[];rows=[]
for key,name in [('gothic','ヒラギノ角ゴシック W3.ttc'),('mincho','ヒラギノ明朝 ProN.ttc')]:fonts.append((key,next(p for p in Path('/System/Library/Fonts').iterdir()if unicodedata.normalize('NFC',p.name)==name)))
with zipfile.ZipFile(r/'corpus.zip','w',compression=zipfile.ZIP_DEFLATED,compresslevel=9)as z:
 def put(name,b):
  i=zipfile.ZipInfo(name,date_time=(1980,1,1,0,0,0));i.compress_type=zipfile.ZIP_DEFLATED;z.writestr(i,b)
 def freeze(id,text,image,meta):
  for fmt in ['png','jpeg75']:
   key=id+'_'+fmt;im='images/'+key+('.png'if fmt=='png'else'.jpg');gt='truth/'+key+'.txt';buf=io.BytesIO();image.save(buf,format='PNG'if fmt=='png'else'JPEG',**({}if fmt=='png'else{'quality':75,'subsampling':2}));b=buf.getvalue();t=text.encode();put(im,b);put(gt,t);rows.append({'id':key,'format':fmt,'truth':text,'width':image.width,'height':image.height,'image_member':im,'truth_member':gt,'image_sha256':sha(b),'truth_sha256':sha(t),**meta})
 def rendered(text,font):
  x0,y0,x1,y1=font.getbbox(text);h=y1-y0+16;im=Image.new('RGB',(max(x1-x0+16,h*2),h),'white');ImageDraw.Draw(im).text((8-x0,8-y0),text,font=font,fill='black');return im
 for font_id,path in fonts:
  font=ImageFont.truetype(str(path),32)
  for body_id,body in [('fee','料金'),('circuit','回路'),('dotted','ijI'),('digits','2048')]:
   for family,c in [('round',')'),('full_round','）'),('corner','」'),('double_corner','』'),('lenticular','】')]:
    for pos in ['prefix','suffix','internal']:
     text=c+body if pos=='prefix'else body+c if pos=='suffix'else body[0]+c+body[1:];freeze(f'glyph_{body_id}_{family}_{pos}_{font_id}',text,rendered(text,font),{'kind':'glyph','body':body,'closer':c,'position':pos,'font':font_id,'size':32})
  for family,c in [('round',')'),('full_round','）'),('corner','」'),('double_corner','』'),('lenticular','】')]:
   for prefix in [False,True]:
    for spaced in [False,True]:
     gap=' 'if spaced else'';body='回路図';text=c+gap+body if prefix else body+gap+c;freeze(f'space_{family}_{"prefix"if prefix else"suffix"}_{"space"if spaced else"joined"}_{font_id}',text,rendered(text,font),{'kind':'space','body':body,'closer':c,'prefix':prefix,'spaced':spaced,'font':font_id,'size':32})
  for body_id,body in [('latin','ABC'),('circuit','回路')]:
   x0,y0,x1,y1=font.getbbox(body);tw,th=x1-x0,y1-y0
   for frames in [1,2]:
    for mark in [False,True]:
     h=th+32;w=max(tw+52,h*2);im=Image.new('RGB',(w,h),'white');d=ImageDraw.Draw(im);d.text((16-x0,16-y0),body,font=font,fill='black');d.rectangle((4,4,tw+28,th+28),outline='black',width=1)
     if frames==2:d.rectangle((8,8,tw+24,th+24),outline='black',width=1)
     if mark:d.rectangle((tw+38,10,tw+39,th+22),fill='black')
     freeze(f'enclosure_{body_id}_f{frames}_mark{int(mark)}_{font_id}',body,im,{'kind':'enclosure','body':body,'frames':frames,'outside_mark':mark,'font':font_id,'size':32})
assert len(rows)==352
m={'font_size':32,'pillow_version':PIL.__version__,'generator_sha256':sha(Path(__file__).read_bytes()),'fonts':[{'id':key,'file':p.name,'sha256':sha(p.read_bytes())}for key,p in fonts],'archive_sha256':sha((r/'corpus.zip').read_bytes()),'rows':rows};(r/'manifest.json').write_text(json.dumps(m,ensure_ascii=False,indent=2)+'\n');print('frozen',len(rows),'images',m['archive_sha256'])
