import cairosvg, os, shutil
from PIL import Image
def G(id,c1,c2,x2="0",y2="1"): return f'<linearGradient id="{id}" x1="0" y1="0" x2="{x2}" y2="{y2}"><stop offset="0" stop-color="{c1}"/><stop offset="1" stop-color="{c2}"/></linearGradient>'
def R(id,c1,c2): return f'<radialGradient id="{id}" cx=".35" cy=".3" r=".8"><stop offset="0" stop-color="{c1}"/><stop offset="1" stop-color="{c2}"/></radialGradient>'
GLOSS=G("gl","#fff","#fff")[:-len('</linearGradient>')].replace('stop-color="#fff"/><stop offset="1" stop-color="#fff"/>','stop-color="#fff" stop-opacity=".85"/><stop offset="1" stop-color="#fff" stop-opacity="0"/>')+'</linearGradient>'
SH='<filter id="sh" x="-20%" y="-20%" width="140%" height="150%"><feDropShadow dx="0" dy="1.5" stdDeviation="1.2" flood-color="#000" flood-opacity=".35"/></filter>'
def svg(defs,body): return f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 48 48" width="48" height="48"><defs>{GLOSS}{SH}{defs}</defs><g filter="url(#sh)">{body}</g></svg>'
OL='stroke-linejoin="round" stroke-linecap="round"'
folder_back=f'<path d="M5 13a3 3 0 0 1 3-3h10l4 4h18a3 3 0 0 1 3 3v21a3 3 0 0 1-3 3H8a3 3 0 0 1-3-3z" fill="url(#fb)" stroke="#9A6A10" stroke-width="1.2" {OL}/>'
fdefs=G("fb","#F7D774","#D9A520")+G("ff","#FFE69A","#E8B02E")
I={}
I["open"]=svg(fdefs,folder_back+f'<path d="M3 21a2 2 0 0 1 2-2h38a2 2 0 0 1 2 2.4l-3 16.6a3 3 0 0 1-3 2.5H8a3 3 0 0 1-3-2.5z" fill="url(#ff)" stroke="#9A6A10" stroke-width="1.2" {OL} transform="skewX(-8) translate(4 0)"/><path d="M9 21h34l-1 6H8z" fill="url(#gl)" opacity=".7" transform="skewX(-8) translate(4 0)"/>')
I["add"]=svg(G("pg","#FFFFFF","#DCE3EC")+R("ad","#8FE36B","#2E8B12"),f'<path d="M9 5h20l8 8v26a2 2 0 0 1-2 2H9a2 2 0 0 1-2-2V7a2 2 0 0 1 2-2z" fill="url(#pg)" stroke="#6B7785" stroke-width="1.2" {OL}/><path d="M29 5v8h8" fill="#C9D2DE" stroke="#6B7785" stroke-width="1.2" {OL}/><path d="M12 18h14M12 23h12M12 28h10" stroke="#9AA6B4" stroke-width="1.6" {OL}/><circle cx="33" cy="34" r="11" fill="url(#ad)" stroke="#1F6A0A" stroke-width="1.2"/><path d="M33 28v12M27 34h12" stroke="#fff" stroke-width="3.4" {OL}/><ellipse cx="33" cy="28.5" rx="8" ry="4.6" fill="url(#gl)" opacity=".8"/>')
I["extract"]=svg(fdefs+R("ex","#7FC4FF","#1157B5"),folder_back.replace("M5 13","M5 13")+f'<path d="M5 19h38v19a3 3 0 0 1-3 3H8a3 3 0 0 1-3-3z" fill="url(#ff)" stroke="#9A6A10" stroke-width="1.2" {OL}/><path d="M6 20h36v6H6z" fill="url(#gl)" opacity=".6"/><path d="M24 4v18M16 15l8 9 8-9" fill="none" stroke="#0D3F87" stroke-width="7" {OL}/><path d="M24 4v18M16 15l8 9 8-9" fill="none" stroke="url(#ex)" stroke-width="4.4" {OL}/>')
I["test"]=svg(R("ts","#9CF07A","#2A8A12"),f'<path d="M24 3l17 6v12c0 11-7.5 19-17 23C14.5 40 7 32 7 21V9z" fill="url(#ts)" stroke="#1D650A" stroke-width="1.4" {OL}/><path d="M24 6l14 5v6c-5 2-23 3-28 1v-7z" fill="url(#gl)" opacity=".75"/><path d="M15 24l6 6 12-13" fill="none" stroke="#fff" stroke-width="4.2" {OL}/>')
I["delete"]=svg(G("bn","#F4F6F8","#A9B3BE","1","0")+R("dx","#FF7A6B","#B5170B"),f'<path d="M10 13l2.5 27a3 3 0 0 0 3 2.8h17a3 3 0 0 0 3-2.8L38 13z" fill="url(#bn)" stroke="#5C6773" stroke-width="1.2" {OL}/><path d="M18 18l1 20M24 18v20M30 18l-1 20" stroke="#7E8994" stroke-width="1.4" {OL}/><rect x="7" y="8" width="34" height="6" rx="2" fill="url(#bn)" stroke="#5C6773" stroke-width="1.2"/><path d="M19 8V5h10v3" fill="none" stroke="#5C6773" stroke-width="1.6" {OL}/><circle cx="35" cy="35" r="9" fill="url(#dx)" stroke="#7E0E05" stroke-width="1.2"/><path d="M31.5 31.5l7 7M38.5 31.5l-7 7" stroke="#fff" stroke-width="3" {OL}/><ellipse cx="35" cy="30.5" rx="6.5" ry="3.6" fill="url(#gl)" opacity=".75"/>')
def orb(id,c1,c2,stk,inner): return svg(R(id,c1,c2),f'<circle cx="24" cy="24" r="19" fill="url(#{id})" stroke="{stk}" stroke-width="1.4"/>{inner}<ellipse cx="24" cy="14" rx="14" ry="8.5" fill="url(#gl)" opacity=".8"/>')
I["info"]=orb("io","#7CC2FF","#0E4FA8","#0A3A7D",f'<circle cx="24" cy="15" r="3" fill="#fff"/><path d="M24 22v13" stroke="#fff" stroke-width="5" {OL}/>')
I["about"]=orb("ab","#7CC2FF","#0E4FA8","#0A3A7D",f'<path d="M18 18.5a6 6 0 1 1 8.5 5.4c-1.8.9-2.5 2-2.5 3.6v1" fill="none" stroke="#fff" stroke-width="4.2" {OL}/><circle cx="24" cy="35" r="2.8" fill="#fff"/>')
I["up"]=svg(R("up","#8FE36B","#2E8B12"),f'<path d="M24 5L41 23H31v18H17V23H7z" fill="url(#up)" stroke="#1D650A" stroke-width="1.4" {OL}/><path d="M24 8L36 21H12z" fill="url(#gl)" opacity=".75"/>')
I["refresh"]=svg(R("rf","#8FE36B","#2E8B12"),f'<path d="M37 18A15 15 0 1 0 39 30" fill="none" stroke="#1D650A" stroke-width="8" {OL}/><path d="M37 18A15 15 0 1 0 39 30" fill="none" stroke="url(#rf)" stroke-width="5.4" {OL}/><path d="M42 7v14H28z" fill="url(#rf)" stroke="#1D650A" stroke-width="1.4" {OL}/>')
I["sfx"]=svg(G("bx","#E9A46A","#A9521C")+G("bt","#F6C79B","#C8642A")+R("bo","#FFF07A","#E8A000"),f'<path d="M6 17l18-8 18 8v18l-18 9-18-9z" fill="url(#bx)" stroke="#6E3510" stroke-width="1.3" {OL}/><path d="M6 17l18 8 18-8-18-8z" fill="url(#bt)" stroke="#6E3510" stroke-width="1.3" {OL}/><path d="M24 25v19" stroke="#6E3510" stroke-width="1.3"/><path d="M6 17l18 8v19L6 35z" fill="#000" opacity=".12"/><path d="M27 1l-8 13h7l-5 12 13-16h-8l5-9z" fill="url(#bo)" stroke="#8A5A00" stroke-width="1.2" {OL}/>')
I["view"]=svg(G("pg","#FFFFFF","#DCE3EC")+G("rim","#5B6470","#23282E")+R("ln","#DFF1FF","#6FB0E8"),f'<path d="M8 4h18l8 8v24a2 2 0 0 1-2 2H8a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2z" fill="url(#pg)" stroke="#6B7785" stroke-width="1.2" {OL}/><path d="M26 4v8h8" fill="#C9D2DE" stroke="#6B7785" stroke-width="1.2" {OL}/><path d="M11 16h14M11 21h10M11 26h8" stroke="#9AA6B4" stroke-width="1.6" {OL}/><path d="M36 36l8 8" stroke="#3A2A14" stroke-width="7" {OL}/><path d="M36 36l8 8" stroke="#C8642A" stroke-width="4.4" {OL}/><circle cx="29" cy="29" r="10" fill="url(#ln)" stroke="url(#rim)" stroke-width="3.2"/><ellipse cx="26.5" cy="25.5" rx="5" ry="3" fill="#fff" opacity=".8"/>')
DIS='<filter id="ds"><feColorMatrix type="saturate" values="0"/><feComponentTransfer><feFuncA type="linear" slope=".45"/></feComponentTransfer></filter>'
for k,v in I.items():
    open(f"svg/{k}.svg","w").write(v)
    d=v.replace("<defs>","<defs>"+DIS,1).replace('<g filter="url(#sh)">','<g filter="url(#ds)">',1)
    open(f"svg/{k}_disabled.svg","w").write(d)
for f in os.listdir("svg"):
    for s in (16,24,32,48): cairosvg.svg2png(url="svg/"+f,write_to=f"png/{f[:-4]}_{s}.png",output_width=s,output_height=s)
names=list(I)
W=Image.new("RGBA",(len(names)*84,200),"#ECE9D8")
for i,n in enumerate(names):
    for row,(nm,sz) in enumerate([(n,48),(n,24),(n+"_disabled",48)]):
        cairosvg.svg2png(url=f"svg/{nm}.svg",write_to="/tmp/p.png",output_width=sz,output_height=sz)
        p=Image.open("/tmp/p.png");W.paste(p,(i*84+42-sz//2,[10,72,128][row]+(48-sz)//2),p)
W.save("/workspace/aero_toolbar_preview.png")
shutil.make_archive("/workspace/shellzip_toolbar_icons_xp","zip",".")
