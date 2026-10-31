import os
C="#C8642A"; D="#2B2F36"; L="#F3D2B8"
H='<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 32 32" width="32" height="32" fill="none" stroke-linecap="round" stroke-linejoin="round">'
S=f'stroke="{D}" stroke-width="2"'
A=f'stroke="{C}" stroke-width="2.4"'
icons={
"open":f'<path d="M4 9a2 2 0 0 1 2-2h6l3 3h10a2 2 0 0 1 2 2v2" {S}/><path d="M4 25V9" {S}/><path d="M4 25l3.5-10a2 2 0 0 1 1.9-1.3H28a1.5 1.5 0 0 1 1.4 2L26 25z" fill="{L}" {S}/>',
"add":f'<rect x="5" y="9" width="18" height="18" rx="3" {S}/><path d="M10 9V7a2 2 0 0 1 2-2h13a2 2 0 0 1 2 2v13a2 2 0 0 1-2 2h-2" {S}/><path d="M14 13v10M9 18h10" {A}/>',
"extract":f'<path d="M4 10a2 2 0 0 1 2-2h6l3 3h11a2 2 0 0 1 2 2v11a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2z" {S}/><path d="M16 14v9M12 19.5l4 4 4-4" {A}/>',
"test":f'<path d="M16 4l10 4v7c0 6.5-4.3 11-10 13-5.7-2-10-6.5-10-13V8z" fill="{L}" {S}/><path d="M11 16l3.5 3.5L21 13" {A}/>',
"delete":f'<path d="M6 9h20M13 9V6h6v3" {S}/><path d="M8 9l1.4 16.2A2 2 0 0 0 11.4 27h9.2a2 2 0 0 0 2-1.8L24 9" {S}/><path d="M13.5 14l5 6M18.5 14l-5 6" {A}/>',
"info":f'<circle cx="16" cy="16" r="12" {S}/><path d="M16 15v8" {A}/><circle cx="16" cy="10" r="1.4" fill="{C}"/>',
"about":f'<circle cx="16" cy="16" r="12" {S}/><path d="M12.5 12.5a3.6 3.6 0 1 1 5 3.3c-1 .5-1.5 1.2-1.5 2.2v.6" {A}/><circle cx="16" cy="22.5" r="1.4" fill="{C}"/>',
"up":f'<path d="M16 26V7M9 14l7-7 7 7" {S}/>',
"refresh":f'<path d="M25 9v5h-5" {A}/><path d="M24.5 14A9 9 0 1 0 25 19" {S}/>',
}
for k,v in icons.items():
    open(f"svg/{k}.svg","w").write(H+v+"</svg>")
    if k in ("extract","delete","test"):
        g=f'<g opacity="0.38" style="filter:grayscale(1)">{v}</g>'
        open(f"svg/{k}_disabled.svg","w").write(H+g.replace(C,"#8A8F98").replace(L,"#E4E6EA").replace(D,"#8A8F98")+"</svg>")
