"""Decode the original player sprite; no new/generated artwork is needed."""
from pathlib import Path
from PIL import Image
r=Path(__file__).resolve().parents[1]
b=(r/'original/PPK.B16').read_bytes(); palette=(r/'original/STAGE.PP3').read_bytes()
attribute=[0,2,4,6,8,10,12,14,1,3,5,7,9,11,13,15]
def color_at(start,x,y):
    return sum(((b[start+p*32+y*2+x//8]>>(7-x%8))&1)<<p for p in range(4))
sprite=Image.new('RGBA',(16,16))
for y in range(16):
    for x in range(16):
        c=attribute[color_at(28*128,x,y)]
        alpha=0 if color_at(48*128+28*128,x,y)==15 else 255
        sprite.putpixel((x,y),tuple(palette[c*3+i]*255//63 for i in range(3))+(alpha,))
for name,size,scale in [('icon-192.png',192,0.7),('icon-512.png',512,0.7),('maskable-512.png',512,0.56),('apple-touch-icon.png',180,0.7)]:
    icon=Image.new('RGB',(size,size),'#16191f');side=round(size*scale)
    actor=sprite.resize((side,side),Image.Resampling.NEAREST)
    icon.paste(actor,((size-side)//2,(size-side)//2),actor)
    icon.save(r/'web/icons'/name)
