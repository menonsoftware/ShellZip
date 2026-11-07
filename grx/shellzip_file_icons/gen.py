import cairosvg, os, io, shutil
from PIL import Image
T={ # name: (label, c1, c2, tar?)
"zip":("ZIP","#FFD54A","#D99A00",0),"7z":("7Z","#5FA8FF","#0E4FA8",0),"rar":("RAR","#C58BFF","#6A1FB5",0),
"tar":("TAR","#C99A6B","#7A4A22",0),"gz":("GZ","#7FE07A","#22861A",0),"zst":("ZST","#5FE0D8","#0E8A85",0),
"xz":("XZ","#FF9A6B","#C2410C",0),"bz2":("BZ2","#FF7A9A","#B5174A",0),
"tar.gz":("TGZ","#7FE07A","#22861A",1),"tar.zst":("TZST","#5FE0D8","#0E8A85",1),"tar.xz":("TXZ","#FF9A6B","#C2410C",1),"tar.bz2":("TBZ2","#FF7A9A","#B5174A",1)}
OL='stroke-linejoin="round" stroke-linecap="round"'
def doc(lbl,c1,c2,tar,small):
    sw=2.2 if small else 1.4
    d=f'''<defs><linearGradient id="pg" x1="0" y1="0" x2="1" y2="1"><stop offset="0" stop-color="#FFFFFF"/><stop offset="1" stop-color="#D5DDE7"/></linearGradient>
<linearGradient id="bd" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="{c1}"/><stop offset="1" stop-color="{c2}"/></linearGradient>
<linearGradient id="tr" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#C99A6B"/><stop offset="1" stop-color="#7A4A22"/></linearGradient>
<linearGradient id="gl" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#fff" stop-opacity=".85"/><stop offset="1" stop-color="#fff" stop-opacity="0"/></linearGradient>
<filter id="sh" x="-20%" y="-20%" width="140%" height="140%"><feDropShadow dx="0" dy="1.5" stdDeviation="1.4" flood-opacity=".35"/></filter></defs>'''
    b=f'<g filter="url(#sh)"><path d="M10 3h20l10 10v31a1.5 1.5 0 0 1-1.5 1.5h-28A1.5 1.5 0 0 1 9 44V4.5A1.5 1.5 0 0 1 10 3z" fill="url(#pg)" stroke="#6B7785" stroke-width="{sw*.8}" {OL}/>'
    b+=f'<path d="M30 3v8.5a1.5 1.5 0 0 0 1.5 1.5H40" fill="#C9D2DE" stroke="#6B7785" stroke-width="{sw*.8}" {OL}/>'
    # zipper strip
    zx=19
    b+=f'<rect x="{zx-2.5}" y="3.6" width="5" height="{18 if not small else 22}" fill="#3B4048"/>'
    for y in range(5,21 if not small else 25,3):
        b+=f'<rect x="{zx-3.5}" y="{y}" width="3" height="1.6" fill="#C0C6CE"/><rect x="{zx+0.5}" y="{y+1.5}" width="3" height="1.6" fill="#C0C6CE"/>'
    pz=22 if not small else 26
    b+=f'<rect x="{zx-3}" y="{pz-1}" width="6" height="7" rx="1.2" fill="#C8642A" stroke="#6E3510" stroke-width=".8"/><rect x="{zx-1.2}" y="{pz+2.5}" width="2.4" height="2" fill="#6E3510"/>'
    if small:
        # big colored band, no text
        if tar: b+=f'<rect x="6" y="34" width="36" height="5" rx="1.5" fill="url(#tr)" stroke="#5A3214" stroke-width="1"/>'
        b+=f'<rect x="6" y="{38 if tar else 33}" width="36" height="{8 if tar else 12}" rx="2" fill="url(#bd)" stroke="{c2}" stroke-width="1.2" filter="brightness(.7)"/>'
    else:
        if tar: b+=f'<rect x="4" y="29" width="40" height="4" rx="1.2" fill="url(#tr)" stroke="#5A3214" stroke-width=".8"/>'
        fs=11 if len(lbl)<=3 else 9
        b+=f'<rect x="4" y="32" width="40" height="11" rx="2" fill="url(#bd)" stroke="{c2}" stroke-width="1"/><rect x="5" y="32.6" width="38" height="4.5" rx="1.5" fill="url(#gl)" opacity=".7"/>'
        b+=f'<text x="24" y="{40.6 if fs==11 else 40.2}" text-anchor="middle" font-family="DejaVu Sans" font-weight="bold" font-size="{fs}" fill="#fff" stroke="{c2}" stroke-width=".5" paint-order="stroke">{lbl}</text>'
    b+='</g>'
    return f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 48 48" width="48" height="48">{d}{b}</svg>'
sizes=[16,20,24,32,40,48,64,96,128,256]
for n,(l,c1,c2,t) in T.items():
    big=doc(l,c1,c2,t,False); sm=doc(l,c1,c2,t,True)
    open(f"svg/{n}.svg","w").write(big); open(f"svg/{n}_small.svg","w").write(sm)
    imgs=[]
    for s in sizes:
        src=sm if s<=24 else big
        im=Image.open(io.BytesIO(cairosvg.svg2png(bytestring=src.encode(),output_width=s,output_height=s))).convert("RGBA")
        im.save(f"png/{n}_{s}.png"); imgs.append(im)
    imgs[-1].save(f"ico/{n}.ico",sizes=[(s,s) for s in sizes],append_images=imgs[:-1])
names=list(T)
W=Image.new("RGBA",(len(names)*84,170),"#FFFFFF")
for i,n in enumerate(names):
    for row,sz in enumerate((64,32,16)):
        p=Image.open(f"png/{n}_{sz}.png");W.paste(p,(i*84+42-sz//2,[6,84,132][row]),p)
W.save("/workspace/fileicons_preview.png")
