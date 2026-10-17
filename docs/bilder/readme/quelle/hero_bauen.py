"""Setzt das echte Bildschirmfoto in die erzeugte Szene und legt Logo und Titel scharf darauf."""
from PIL import Image, ImageDraw, ImageFont, ImageFilter, ImageEnhance
import numpy as np

F = "/usr/share/fonts/opentype/inter/"
GOLD, WEISS, GRAU = (196, 155, 24), (245, 243, 238), (201, 206, 216)

szene = Image.open("hero-roh-0.png").convert("RGB")
a = np.array(szene).astype(int)
gruen = (a[:, :, 1] > 150) & (a[:, :, 1] > a[:, :, 0] + 60) & (a[:, :, 1] > a[:, :, 2] + 60)
ys, xs = np.nonzero(gruen)
x0, x1, y0, y1 = xs.min(), xs.max() + 1, ys.min(), ys.max() + 1
w, h = x1 - x0, y1 - y0

# Bildschirmfoto auf Breite, oben und unten mit der eigenen Randfarbe auffüllen.
foto = Image.open("03-schnellstatus.png").convert("RGB")
s = foto.resize((w, round(foto.height * w / foto.width)), Image.LANCZOS)
schirm = Image.new("RGB", (w, h), s.getpixel((w // 2, 2)))
oben = (h - s.height) // 2
schirm.paste(s, (0, oben))
unten = Image.new("RGB", (w, h - oben - s.height), s.getpixel((w // 2, s.height - 2)))
schirm.paste(unten, (0, oben + s.height))
schirm = ImageEnhance.Brightness(schirm).enhance(0.9)
# leichter warmer Schimmer wie das Morgenlicht von rechts
warm = Image.new("RGB", (w, h), (255, 214, 150))
verlauf = Image.linear_gradient("L").rotate(90).resize((w, h))
schirm = Image.composite(Image.blend(schirm, warm, 0.10), schirm, verlauf)

maske = Image.fromarray((gruen[y0:y1, x0:x1] * 255).astype("uint8")).filter(ImageFilter.MaxFilter(5))
szene.paste(schirm, (x0, y0), maske)

# grünen Saum und Spiegelung auf dem Tisch entfärben
a = np.array(szene).astype(int)
bereich = np.zeros(gruen.shape, bool)
bereich[max(y0 - 8, 0):min(y1 + 300, a.shape[0]), max(x0 - 60, 0):min(x1 + 30, a.shape[1])] = True
bereich[y0 + 6:y1 - 6, x0 + 6:x1 - 6] = False
r, g, b = a[:, :, 0], a[:, :, 1], a[:, :, 2]
stich = bereich & (g > r + 8) & (g > b + 8)
a[:, :, 1] = np.where(stich, np.maximum(r, b), g)
szene = Image.fromarray(a.clip(0, 255).astype("uint8"))

# Ausschnitt 2:1
oben_y = 192
szene = szene.crop((0, oben_y, 1536, oben_y + 768))
d = ImageDraw.Draw(szene)

logo = Image.open("logo-gold-transparent.png").convert("RGBA")
logo = logo.crop(logo.getbbox())
lh = 64
logo = logo.resize((round(logo.width * lh / logo.height), lh), Image.LANCZOS)
szene.paste(logo, (88, 70), logo)
d.text((88 + logo.width + 18, 70 + lh // 2), "Blackfort Technology", font=ImageFont.truetype(F + "Inter-Medium.otf", 26), fill=GOLD, anchor="lm")

d.text((84, 300), "Days & Keys", font=ImageFont.truetype(F + "Inter-Bold.otf", 104), fill=WEISS, anchor="ls")
d.text((88, 372), "Festhalten, wie es geht.", font=ImageFont.truetype(F + "Inter-SemiBold.otf", 42), fill=GOLD, anchor="ls")
klein = ImageFont.truetype(F + "Inter-Regular.otf", 29)
d.text((88, 450), "Ein privates Tagebuch für Stimmung, Schlaf,", font=klein, fill=GRAU, anchor="ls")
d.text((88, 492), "Medikamente und das, was den Tag prägt.", font=klein, fill=GRAU, anchor="ls")
mittel = ImageFont.truetype(F + "Inter-Medium.otf", 26)
d.text((88, 560), "Verschlüsselt auf dem eigenen Gerät.", font=mittel, fill=WEISS, anchor="ls")
d.text((88, 598), "Ohne Konto, ohne Server.", font=mittel, fill=WEISS, anchor="ls")

szene.save("hero.jpg", quality=92, optimize=True, progressive=True)
print("hero.jpg", szene.size, "Schirm", x0, y0, w, h)
