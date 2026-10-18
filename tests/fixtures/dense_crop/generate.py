from pathlib import Path
import hashlib,json,unicodedata,PIL
from PIL import Image,ImageDraw,ImageFont
r=Path('tests/fixtures/dense_crop');sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest()
fonts=[]
for label,name in [('thin','ヒラギノ角ゴシック W2.ttc'),('heavy','ヒラギノ角ゴシック W8.ttc')]:
 p=next(p for p in Path('/System/Library/Fonts').iterdir() if unicodedata.normalize('NFC',p.name)==name);fonts.append((label,p))
texts={'ja':'公開前の資料は複数の担当者が読み、表と本文の番号、注記の位置、日付の表記などを照合します。画面で見つけにくい箇所は印刷して確認し、修正した内容と確認した時刻を記録します。作業が終わったら、変更したファイルを保存し、古い版との違いを説明した一覧を添えて担当部署へ送ります。', 'en':'Before publication, several readers compare the table numbers, dates, notes, and references in each document. They record their findings, verify the revised files, and prepare a short description of the changes for the next team.'}
rows=[]
for label,path in fonts:
 for size in [20,40]:
  font=ImageFont.truetype(str(path),size)
  for language,text in texts.items():
   x0,y0,x1,y1=font.getbbox(text)
   for margin in [0,6]:
    im=Image.new('RGB',(x1-x0+8,y1-y0+margin*2),'white');ImageDraw.Draw(im).text((4-x0,margin-y0),text,font=font,fill='black')
    for form in ['png','jpg']if margin==0 else ['png']:
     stem=f'{language}_{label}{size}_pad{margin}'+('_jpeg75'if form=='jpg'else'');image=r/'images'/(stem+'.'+form);truth=r/'truth'/(stem+'.txt')
     if form=='jpg':im.save(image,quality=75,subsampling=2)
     else:im.save(image)
     truth.write_text(text);rows.append({'id':stem,'image':str(image),'truth':text,'image_sha256':sha(image),'truth_sha256':sha(truth),'font':label,'font_size':size,'vertical_margin':margin,'format':form})
(r/'manifest.json').write_text(json.dumps({'pillow_version':PIL.__version__,'fonts':[{'label':n,'name':p.name,'sha256':sha(p)}for n,p in fonts],'rows':rows},ensure_ascii=False,indent=2)+'\n');print('frozen',len(rows))
