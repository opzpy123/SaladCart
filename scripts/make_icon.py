#!/usr/bin/env python3
"""Generate the SaladCart master icon (1024x1024 PNG) with PIL."""
from PIL import Image, ImageDraw
import math

S = 4096
img = Image.new("RGBA", (S, S), (0, 0, 0, 0))
px = img.load()

# Rounded-square background with a subtle diagonal gradient.
radius = int(S * 0.225)
for y in range(S):
    for x in range(S):
        t = (x / S * 0.55 + y / S * 0.45)
        r = int(23 + 18 * t)
        g = int(27 + 30 * t)
        b = int(21 + 9 * t)
        px[x, y] = (r, g, b, 255)

mask = Image.new("L", (S, S), 0)
md = ImageDraw.Draw(mask)
md.rounded_rectangle([0, 0, S - 1, S - 1], radius=radius, fill=255)
img.putalpha(mask)
draw = ImageDraw.Draw(img)

cream = (246, 242, 226, 255)
lime = (166, 208, 80, 255)
tomato = (228, 93, 51, 255)
ink = (23, 25, 19, 255)
pale = (203, 218, 176, 255)

unit = S / 1024.0

def u(v):
    return int(v * unit)

# Film-frame basket
left, right = u(232), u(792)
top, bottom = u(300), u(690)
stroke = u(34)
draw.rounded_rectangle([left, top, right, bottom], radius=u(58), outline=cream, width=stroke)
# Open basket top: erase center of top edge and draw handles
draw.rectangle([left + u(40), top - stroke, right - u(40), top + stroke], fill=(0, 0, 0, 0))
draw.rounded_rectangle([left + u(18), top + u(30), right - u(18), bottom], radius=u(46), outline=cream, width=stroke)
# Handle film slots
for x in [left + u(90), left + u(190), left + u(290)]:
    draw.rounded_rectangle([x, top + u(66), x + u(58), top + u(110)], radius=u(12), fill=ink)
for x in [right - u(148), right - u(248), right - u(348)]:
    draw.rounded_rectangle([x, top + u(66), x + u(58), top + u(110)], radius=u(12), fill=ink)

# Cart handle
draw.arc([u(690), u(150), u(980), u(480)], start=200, end=345, fill=cream, width=stroke)
draw.line([u(760), u(435), u(760), u(520)], fill=cream, width=stroke)

# Salad inside the basket
draw.ellipse([u(360), u(410), u(530), u(580)], fill=lime)
draw.ellipse([u(490), u(380), u(665), u(555)], fill=tomato)
draw.ellipse([u(540), u(510), u(700), u(670)], fill=pale)
draw.ellipse([u(350), u(530), u(500), u(680)], fill=(120, 164, 80, 255))
# Highlights
draw.ellipse([u(398), u(448), u(446), u(496)], fill=(214, 238, 150, 255))
draw.ellipse([u(528), u(418), u(566), u(456)], fill=(247, 160, 128, 255))

# Wheels
for cx in (u(360), u(690)):
    draw.ellipse([cx - u(62), u(740) - u(62), cx + u(62), u(740) + u(62)], fill=lime, outline=ink, width=u(12))
    draw.ellipse([cx - u(22), u(740) - u(22), cx + u(22), u(740) + u(22)], fill=ink)

# Little spark / signal
draw.rounded_rectangle([u(790), u(190), u(840), u(240)], radius=u(12), fill=tomato)

# Downsample for clean edges
out = img.resize((1024, 1024), Image.LANCZOS)
out.save("assets/brand/icon-source.png")
print("wrote assets/brand/icon-source.png")
