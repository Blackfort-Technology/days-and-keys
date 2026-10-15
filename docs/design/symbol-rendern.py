import math
from PIL import Image, ImageDraw
import os
D = os.path.join(os.path.dirname(os.path.abspath(__file__)), "")
PETROL=(0x1E,0x5F,0x5A); SAND=(0xF3,0xE9,0xD8); OCKER=(0xE3,0xB5,0x6C)
# Geometrie in dp auf der 108-dp-Ebene (gemessen an symbol-a2-horizont-1, Spalt verbreitert)
CX, CY, R = 54.0, 54.0, 27.0
OBEN, UNTEN = 54.25, 56.75     # Horizontspalt
RS = 13.0                      # Sonne, Halbkreis auf OBEN
LUECKE = 2.25                  # Spalt um die Sonne in der Monochrom-Ebene
def ebene(px, farben=True, mono=False, skala=1.0, hintergrund=None):
    ss = 4; n = px * ss; f = n / 108.0 * skala; off = (n - 108 * f) / 2
    P = lambda x, y: (off + x * f, off + y * f)
    img = Image.new("RGBA", (n, n), (hintergrund + (255,)) if hintergrund else (0, 0, 0, 0))
    m = Image.new("L", (n, n), 0); d = ImageDraw.Draw(m)
    x0, y0 = P(CX - R, CY - R); x1, y1 = P(CX + R, CY + R); d.ellipse((x0, y0, x1, y1), fill=255)
    _, g0 = P(0, OBEN); _, g1 = P(0, UNTEN); d.rectangle((0, g0, n, g1), fill=0)
    sx0, sy0 = P(CX - RS, OBEN - RS); sx1, sy1 = P(CX + RS, OBEN + RS)
    if mono:
        lx0, ly0 = P(CX - RS - LUECKE, OBEN - RS - LUECKE); lx1, ly1 = P(CX + RS + LUECKE, OBEN + RS + LUECKE)
        d.pieslice((lx0, ly0, lx1, ly1), 180, 360, fill=0)
        d.pieslice((sx0, sy0, sx1, sy1), 180, 360, fill=255)
        img.paste((255, 255, 255, 255), (0, 0), m)
    else:
        img.paste(SAND + (255,), (0, 0), m)
        s = Image.new("L", (n, n), 0); ImageDraw.Draw(s).pieslice((sx0, sy0, sx1, sy1), 180, 360, fill=255)
        img.paste(OCKER + (255,), (0, 0), s)
    return img.resize((px, px), Image.LANCZOS)
ebene(1024).save(D + "vordergrund-freigestellt.png")
ebene(1024, mono=True).save(D + "monochrom.png")
ebene(1024, hintergrund=PETROL).convert("RGB").save(D + "symbol-gewaehlt-1024.png")
# iOS: kein Sicherheitsrand, Scheibe auf 64 % der Kantenlänge
k = 0.64 / 0.5
ebene(1024, skala=k, hintergrund=PETROL).convert("RGB").save(D + "ios/AppIcon-1024.png")
ebene(1024, skala=k).save(D + "ios/AppIcon-dunkel-1024.png")          # transparenter Grund, System legt den dunklen Grund
g = ebene(1024, skala=k, hintergrund=(0, 0, 0)).convert("L"); g.save(D + "ios/AppIcon-getoent-1024.png")
# Play-Store-Grafik 512
ebene(512, hintergrund=PETROL).convert("RGB").save(D + "android/play-store-512.png")
print("ok")
