from pathlib import Path
from PIL import Image, ImageDraw

root=Path(__file__).resolve().parent/'rendered-action-pass'
pages=sorted(root.glob('page-*.png'), key=lambda p:int(p.stem.split('-')[1]))
out=root/'contacts'; out.mkdir(exist_ok=True)
for batch in range(0,len(pages),6):
    chunk=pages[batch:batch+6]; thumbs=[]
    for p in chunk:
        im=Image.open(p).convert('RGB'); im.thumbnail((520,680))
        canvas=Image.new('RGB',(540,720),'white'); canvas.paste(im,((540-im.width)//2,28))
        ImageDraw.Draw(canvas).text((12,8),p.stem,fill='black'); thumbs.append(canvas)
    sheet=Image.new('RGB',(1080,2160),(210,210,210))
    for i,im in enumerate(thumbs): sheet.paste(im,((i%2)*540,(i//2)*720))
    sheet.save(out/f'contact-{batch//6+1}.png')
