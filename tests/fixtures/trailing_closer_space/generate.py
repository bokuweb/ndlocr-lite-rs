"""Freeze supplementary controls before inspecting the trailing-space candidate."""
from pathlib import Path
import json,hashlib,io,unicodedata,zipfile
import PIL
from PIL import Image,ImageDraw,ImageFont
r=Path(__file__).resolve().parent;sha=lambda b:hashlib.sha256(b).hexdigest();fonts=[];rows=[]
for id,name in [('maru','ヒラギノ丸ゴ ProN W4.ttc'),('gothic','ヒラギノ角ゴシック W3.ttc')]:fonts.append((id,next(p for p in Path('/System/Library/Fonts').iterdir()if unicodedata.normalize('NFC',p.name)==name)))
with zipfile.ZipFile(r/'corpus.zip','w',compression=zipfile.ZIP_DEFLATED,compresslevel=9)as z:
 def put(name,b):
  i=zipfile.ZipInfo(name,date_time=(1980,1,1,0,0,0));i.compress_type=zipfile.ZIP_DEFLATED;z.writestr(i,b)
 def freeze(id,text,image,metadata):
  for fmt in ['png','jpeg75']:
   key=id+'_'+fmt;im='images/'+key+('.png'if fmt=='png'else'.jpg');gt='truth/'+key+'.txt';buf=io.BytesIO()
   if fmt=='png':image.save(buf,format='PNG')
   else:image.save(buf,format='JPEG',quality=75,subsampling=2)
   data,truth=buf.getvalue(),text.encode();put(im,data);put(gt,truth);rows.append({'id':key,'format':fmt,'truth':text,'width':image.width,'height':image.height,'image_member':im,'truth_member':gt,'image_sha256':sha(data),'truth_sha256':sha(truth),**metadata})
 for font_id,font_path in fonts:
  for size in [20,40]:
   font=ImageFont.truetype(str(font_path),size)
   for family,close in [('round',')'),('full_round','）'),('corner','」'),('double_corner','』'),('lenticular','】')]:
    for prefix in [False,True]:
     for spaced in [False,True]:
      gap=' 'if spaced else'';text=close+gap+'DEF'if prefix else'DEF'+gap+close;x0,y0,x1,y1=font.getbbox(text);im=Image.new('RGB',(x1-x0+16,y1-y0+16),'white');ImageDraw.Draw(im).text((8-x0,8-y0),text,font=font,fill='black');id=f'{family}_{"prefix"if prefix else"suffix"}_{"space"if spaced else"joined"}_{font_id}{size}';freeze(id,text,im,{'kind':'text','closer':close,'prefix':prefix,'spaced':spaced,'font':font_id,'size':size})
 font_path=next(path for id,path in fonts if id=='gothic');font=ImageFont.truetype(str(font_path),28);x0,y0,x1,y1=font.getbbox('DEF');tw,th=x1-x0,y1-y0
 for width in [1,2]:
  for gap in [2,10]:
   for height in [22,66]:
    canvas_h=max(th,height)+16;im=Image.new('RGB',(max(tw+gap+width+16,canvas_h*2),canvas_h),'white');d=ImageDraw.Draw(im);d.text((8-x0,8-y0),'DEF',font=font,fill='black');d.rectangle((8+tw+gap,8,8+tw+gap+width-1,8+height-1),fill='black');freeze(f'rule_w{width}_g{gap}_h{height}','DEF',im,{'kind':'rule','rule_width':width,'rule_gap':gap,'rule_height':height,'font':'gothic','size':28})
assert len(rows)==176
m={'sizes':[20,40],'margin':8,'pillow_version':PIL.__version__,'generator_sha256':sha(Path(__file__).read_bytes()),'fonts':[{'id':id,'file':p.name,'sha256':sha(p.read_bytes())}for id,p in fonts],'archive_sha256':sha((r/'corpus.zip').read_bytes()),'rows':rows};(r/'manifest.json').write_text(json.dumps(m,ensure_ascii=False,indent=2)+'\n',encoding='utf-8');print('frozen',len(rows),'images',m['archive_sha256'])
