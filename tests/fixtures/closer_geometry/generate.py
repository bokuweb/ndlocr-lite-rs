"""Freeze new closer geometry and spacing controls before any model inspection."""
from pathlib import Path
import hashlib, io, json, unicodedata, zipfile
import PIL
from PIL import Image, ImageDraw, ImageFont
root=Path(__file__).resolve().parent
sha=lambda data:hashlib.sha256(data).hexdigest()
fonts=[]
for key,name in [('maru','ヒラギノ丸ゴ ProN W4.ttc'),('mincho','ヒラギノ明朝 ProN.ttc')]:
 path=next(p for p in Path('/System/Library/Fonts').iterdir()if unicodedata.normalize('NFC',p.name)==name);fonts.append((key,path))
cases=[]
for family,opening,closing in [('round','(',')'),('full_round','（','）'),('corner','「','」'),('double_corner','『','』'),('lenticular','【','】')]:
 for body_id,body in [('latin','Agjp'),('digits','2026'),('japanese','情報')]:
  for position,text in [('prefix',closing+body),('suffix',body+closing),('internal',body[:1]+closing+body[1:])]:cases.append((f'{family}_{body_id}_{position}',text))
 cases.extend([(family+'_balanced',opening+'情報'+closing),(family+'_alone',closing),(family+'_double','Agjp'+closing*2)])
for i,text in enumerate(['Agjp','情報','2026','Ag jp','情報 処理','A.B','i j','j i','一 二','三 川','Agjp.','Agjp,','Agjp!','Agjp?','情報。','情報、','A) gjp','A） gjp','A』 gjp','） 情報']):cases.append((f'control_{i:02}',text))
assert len(cases)==len(dict(cases))==80
rows=[]
with zipfile.ZipFile(root/'corpus.zip','w',compression=zipfile.ZIP_DEFLATED,compresslevel=9)as archive:
 def put(name,data):
  info=zipfile.ZipInfo(name,date_time=(1980,1,1,0,0,0));info.compress_type=zipfile.ZIP_DEFLATED;archive.writestr(info,data)
 def freeze(case,text,image,font_id,size):
  for fmt in ['png','jpeg75']:
   id=f'{case}_{font_id}{size}_{fmt}';im='images/'+id+('.png'if fmt=='png'else'.jpg');gt='truth/'+id+'.txt';buffer=io.BytesIO()
   if fmt=='png':image.save(buffer,format='PNG')
   else:image.save(buffer,format='JPEG',quality=75,subsampling=2)
   data,truth=buffer.getvalue(),text.encode();put(im,data);put(gt,truth);rows.append({'id':id,'case':case,'font':font_id,'size':size,'format':fmt,'width':image.width,'height':image.height,'truth':text,'image_member':im,'truth_member':gt,'image_sha256':sha(data),'truth_sha256':sha(truth)})
 for font_id,font_path in fonts:
  for size in [20,40]:
   font=ImageFont.truetype(str(font_path),size)
   for case,text in cases:
    x0,y0,x1,y1=font.getbbox(text);image=Image.new('RGB',(x1-x0+16,y1-y0+16),'white');ImageDraw.Draw(image).text((8-x0,8-y0),text,font=font,fill='black');freeze(case,text,image,font_id,size)
 for kind in ['horizontal','vertical','rectangle','grid','diagonal','low_frame','dots','two_rows']:
  image=Image.new('RGB',(112,64),'white');d=ImageDraw.Draw(image)
  if kind=='horizontal':d.line((12,32,100,32),fill='black',width=2)
  elif kind=='vertical':d.line((56,10,56,54),fill='black',width=2)
  elif kind=='rectangle':d.rectangle((12,10,100,54),outline='black',width=2)
  elif kind=='grid':
   for x in [20,45,70,95]:d.line((x,10,x,54),fill='black',width=2)
   for y in [10,32,54]:d.line((20,y,95,y),fill='black',width=2)
  elif kind=='diagonal':d.line((15,54,98,10),fill='black',width=3)
  elif kind=='low_frame':d.line([(15,18),(15,50),(100,50),(100,18)],fill='black',width=2)
  elif kind=='dots':
   for x in range(18,98,16):d.ellipse((x,31,x+2,33),fill='black')
  elif kind=='two_rows':
   for y in [10,44]:
    for x in [15,35,55,75,95]:d.rectangle((x,y,x+5,y+9),fill='black')
  freeze('figure_'+kind,'',image,'none',0)
assert len(rows)==656
manifest={'font_sizes':[20,40],'margin':8,'pillow_version':PIL.__version__,'generator_sha256':sha(Path(__file__).read_bytes()),'fonts':[{'id':key,'file':path.name,'sha256':sha(path.read_bytes())}for key,path in fonts],'archive_sha256':sha((root/'corpus.zip').read_bytes()),'rows':rows};(root/'manifest.json').write_text(json.dumps(manifest,ensure_ascii=False,indent=2)+'\n',encoding='utf-8');print('frozen',len(rows),'images; archive',manifest['archive_sha256'])
