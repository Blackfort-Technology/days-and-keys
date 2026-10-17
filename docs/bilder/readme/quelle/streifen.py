"""Bildstreifen mit echten Bildschirmfotos (erfundene Daten) im Blackfort-Stil für die README."""
import sys
from PIL import Image, ImageDraw, ImageFont, ImageFilter

F = "/usr/share/fonts/opentype/inter/"
NAVY1, NAVY2, GOLD, WEISS, GRAU = (7, 16, 31), (14, 34, 70), (196, 155, 24), (245, 243, 238), (170, 180, 196)
B, R, ABST, RAND = 300, 34, 56, 64  # Telefonbreite, Eckenradius, Abstand, Rand


def telefon(pfad):
    f = Image.open(pfad).convert("RGB")
    f = f.resize((B, round(f.height * B / f.width)), Image.LANCZOS)
    rahmen = 9
    t = Image.new("RGBA", (B + 2 * rahmen, f.height + 2 * rahmen), (0, 0, 0, 0))
    ImageDraw.Draw(t).rounded_rectangle((0, 0, t.width - 1, t.height - 1), R + rahmen, fill=(18, 20, 24, 255))
    m = Image.new("L", f.size, 0)
    ImageDraw.Draw(m).rounded_rectangle((0, 0, B - 1, f.height - 1), R, fill=255)
    t.paste(f, (rahmen, rahmen), m)
    return t


def streifen(ziel, eintraege):
    tel = [telefon(p) for p, _, _ in eintraege]
    tw, th = tel[0].size
    breite = 2 * RAND + len(tel) * tw + (len(tel) - 1) * ABST
    hoehe = RAND + th + 150
    bild = Image.linear_gradient("L").resize((breite, hoehe))
    bild = Image.composite(Image.new("RGB", (breite, hoehe), NAVY1), Image.new("RGB", (breite, hoehe), NAVY2), bild)
    schatten = Image.new("L", (breite, hoehe), 0)
    sd = ImageDraw.Draw(schatten)
    for i in range(len(tel)):
        x = RAND + i * (tw + ABST)
        sd.rounded_rectangle((x + 6, RAND + 18, x + tw + 6, RAND + th + 18), R + 9, fill=150)
    bild.paste((0, 0, 0), (0, 0), schatten.filter(ImageFilter.GaussianBlur(18)))
    d = ImageDraw.Draw(bild)
    titel, zeile = ImageFont.truetype(F + "Inter-SemiBold.otf", 30), ImageFont.truetype(F + "Inter-Regular.otf", 23)
    for i, (t, (_, kopf, unter)) in enumerate(zip(tel, eintraege)):
        x = RAND + i * (tw + ABST)
        bild.paste(t, (x, RAND), t)
        d.text((x + tw // 2, RAND + th + 52), kopf, font=titel, fill=GOLD, anchor="ms")
        d.text((x + tw // 2, RAND + th + 90), unter, font=zeile, fill=GRAU, anchor="ms")
    bild.save(ziel, quality=92, optimize=True, progressive=True)
    print(ziel, bild.size)


streifen("festhalten.jpg", [
    ("05-status.png", "Status", "so viel, wie gerade passt"),
    ("08-schlaf.png", "Schlaf", "Stunden und Güte 1 bis 5"),
    ("14-praeparat.png", "Medikamente", "Präparat und Einnahmeplan"),
    ("07-tagesabschluss.png", "Tagesabschluss", "abends, freiwillig"),
])
streifen("zurueckschauen.jpg", [
    ("09-verlauf.png", "Verlauf", "Punkte und Grundniveau"),
    ("11-eintraege.png", "Einträge", "nach Tagen geordnet"),
    ("16-gesperrt.png", "Gesperrt", "Fingerabdruck oder Code"),
    ("17-hilfe.png", "Hilfe", "Notruf zuerst, auch gesperrt"),
])
