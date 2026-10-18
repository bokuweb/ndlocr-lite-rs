"""Freeze all repeated-glyph conditions before model inspection; fonts stay local."""
from pathlib import Path
import hashlib,io,json,unicodedata,zipfile
import PIL
from PIL import Image,ImageDraw,ImageFont
root=Path(__file__).resolve().parent
sha=lambda b:hashlib.sha256(b).hexdigest()
fonts=[]
for key,name in [('gothic','ヒラギノ角ゴシック W3.ttc'),('mincho','ヒラギノ明朝 ProN.ttc')]:
 path=next(p for p in Path('/System/Library/Fonts').iterdir()if unicodedata.normalize('NFC',p.name)==name)
 fonts.append((key,path))
units=[('one','1'),('zero','0'),('eight','8'),('upper_i','I'),('lower_l','l'),('upper_o','O'),('kanji_one','一'),('mixed_a1','A1')]
rows=[]
with zipfile.ZipFile(root/'corpus.zip','w',compression=zipfile.ZIP_DEFLATED,compresslevel=9)as archive:
 def put(name,data):
  info=zipfile.ZipInfo(name,date_time=(1980,1,1,0,0,0));info.compress_type=zipfile.ZIP_DEFLATED;archive.writestr(info,data)
 for font_key,font_path in fonts:
  font=ImageFont.truetype(str(font_path),28)
  for label,unit in units:
   for count in [2,4,6,8,12]:
    text=unit*count;x0,y0,x1,y1=font.getbbox(text);im=Image.new('RGB',(x1-x0+16,y1-y0+16),'white');ImageDraw.Draw(im).text((8-x0,8-y0),text,font=font,fill='black')
    for fmt in ['png','jpeg75']:
     stem=f'{label}_n{count:02d}_{font_key}_{fmt}';image_member='images/'+stem+('.png'if fmt=='png'else'.jpg');truth_member='truth/'+stem+'.txt';buffer=io.BytesIO()
     if fmt=='png':im.save(buffer,format='PNG')
     else:im.save(buffer,format='JPEG',quality=75,subsampling=2)
     data=buffer.getvalue();truth=text.encode('utf-8');put(image_member,data);put(truth_member,truth)
     rows.append({'id':stem,'unit':unit,'repeat_count':count,'font':font_key,'format':fmt,'width':im.width,'height':im.height,'truth':text,'image_member':image_member,'truth_member':truth_member,'image_sha256':sha(data),'truth_sha256':sha(truth)})
manifest={'font_size':28,'margin':8,'pillow_version':PIL.__version__,'generator_sha256':sha(Path(__file__).read_bytes()),'fonts':[{'id':key,'file':path.name,'sha256':sha(path.read_bytes())}for key,path in fonts],'archive_sha256':sha((root/'corpus.zip').read_bytes()),'rows':rows}
(root/'manifest.json').write_text(json.dumps(manifest,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
print('frozen',len(rows),'images; archive',manifest['archive_sha256'])
