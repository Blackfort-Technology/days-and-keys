"""Titelbild für die README von Days & Keys: Szene mit grünem Telefonbildschirm zum Einsetzen."""
import base64, sys
from openai import OpenAI
PROMPT = (
    "Realistic editorial photograph, calm early morning at home. A small dark wooden side table in front of "
    "a deep navy blue painted wall. Warm golden sunrise light falls softly from the right through a window "
    "out of frame, creating a gentle golden glow on the wall and table edge. On the right third of the image "
    "a modern smartphone stands upright in a simple minimal stand, screen facing the camera straight on, "
    "perfectly frontal, not tilted. The entire phone display is a uniform flat pure chroma green (#00FF00), "
    "edge to edge, no reflections, no glare, no content. Next to it a plain ceramic cup of tea and a small "
    "closed paper notebook with a pencil. The left half of the image is calm, mostly empty dark navy wall "
    "with a soft gradient, leaving generous space for a title. Colour palette: deep navy (#07101F, #0E2246) "
    "with warm gold accents (#C49B18). Quiet, trustworthy, premium, realistic scale, shallow depth of field. "
    "No people, no hands, no text, no letters, no logos, no icons, no floating objects, no collage."
)
c = OpenAI()
r = c.images.generate(model="gpt-image-2.5-flare", prompt=PROMPT, size="1536x1024", quality="high", n=int(sys.argv[1]) if len(sys.argv) > 1 else 2)
for i, d in enumerate(r.data):
    open(f"hero-roh-{i}.png", "wb").write(base64.b64decode(d.b64_json))
    print("hero-roh-%d.png" % i)
print(getattr(r, "usage", None))
