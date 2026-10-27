#!/usr/bin/env python3
"""Generate offline demo posters, beat track and clips for the browser bridge."""
import math
import os
import random
import struct
import subprocess
import wave
from pathlib import Path

from PIL import Image, ImageDraw, ImageFont

ROOT = Path(__file__).resolve().parent.parent
OUT = ROOT / "public" / "demo"
OUT.mkdir(parents=True, exist_ok=True)
FFMPEG = os.environ.get("FFMPEG_BIN", "ffmpeg")

PALETTES = [
    ((13, 23, 18), (55, 78, 45), "#b5d464", "RAIN / THE FIRST SIGNAL", "Fog Harbor · Episode 01"),
    ((24, 15, 12), (91, 43, 31), "#e58b5b", "THE OLD KEY", "A memory that should not exist"),
    ((12, 20, 27), (31, 61, 74), "#74c6bd", "LAST TRAIN", "Whatever the glass remembers"),
]


def font(size: int):
    for path in [
        "/System/Library/Fonts/Supplemental/Arial.ttf",
        "/System/Library/Fonts/Helvetica.ttc",
        "/Library/Fonts/Arial.ttf",
    ]:
        if Path(path).exists():
            try:
                return ImageFont.truetype(path, size)
            except OSError:
                pass
    return ImageFont.load_default()


def make_poster(index: int) -> Path:
    bg1, bg2, accent, title, sub = PALETTES[index]
    W, H = 720, 1280
    img = Image.new("RGB", (W, H))
    px = img.load()
    for y in range(H):
        for x in range(W):
            t = (x / W) * 0.45 + (y / H) * 0.55
            r = int(bg1[0] + (bg2[0] - bg1[0]) * t)
            g = int(bg1[1] + (bg2[1] - bg1[1]) * t)
            b = int(bg1[2] + (bg2[2] - bg1[2]) * t)
            glow = max(0.0, 1 - math.hypot(x - W * 0.58, y - H * 0.36) / (W * 0.75))
            r = min(255, int(r + glow * 75))
            g = min(255, int(g + glow * 55))
            b = min(255, int(b + glow * 22))
            n = random.randint(-5, 5)
            px[x, y] = (max(0, r + n), max(0, g + n), max(0, b + n))

    draw = ImageDraw.Draw(img, "RGBA")
    rnd = random.Random(31 + index)
    for _ in range(48):
        x = rnd.randint(0, W)
        draw.line([(x, 0), (x - rnd.randint(30, 90), H)], fill=(240, 245, 230, 12), width=rnd.randint(1, 3))
    draw.ellipse([W * 0.25, H * 0.2, W * 0.95, H * 0.72], fill=(*ImageColor(accent), 18))
    # silhouette
    cx, cy = W * 0.52, H * 0.47
    draw.ellipse([cx - 76, cy - 92, cx + 76, cy + 92], fill=(8, 11, 8, 235))
    draw.polygon(
        [(cx - 190, H), (cx - 145, cy + 100), (cx + 145, cy + 100), (cx + 190, H)],
        fill=(8, 11, 8, 240),
    )
    draw.arc([cx - 78, cy - 92, cx + 78, cy + 92], 205, 335, fill=accent, width=4)
    draw.text((44, 52), "SALADCART · DEMO FOOTAGE", font=font(16), fill=(230, 238, 218, 210))
    draw.text((44, 78), f"TAKE {index + 1:02d} / MOCK PROVIDER", font=font(12), fill=(*ImageColor(accent), 200))
    draw.rectangle([0, H - 250, W, H], fill=(7, 9, 6, 205))
    draw.text((44, H - 200), title, font=font(42), fill=(246, 242, 230, 255))
    draw.text((44, H - 138), sub, font=font(17), fill=(*ImageColor(accent), 240))
    draw.text((44, H - 96), "720 × 1280  ·  5.0s  ·  NO AUDIO", font=font(13), fill=(150, 160, 140, 220))
    path = OUT / f"poster-{index + 1}.png"
    img.save(path)
    return path


def ImageColor(value: str):
    value = value.lstrip("#")
    return tuple(int(value[i : i + 2], 16) for i in (0, 2, 4))


def make_score() -> Path:
    rate = 44100
    duration = 24.0
    bpm = 100.0
    beat = 60.0 / bpm
    total = int(rate * duration)
    samples = [0.0] * total
    chords = [
        [220.00, 261.63, 329.63, 392.00],
        [174.61, 220.00, 261.63, 329.63],
        [130.81, 196.00, 261.63, 329.63],
        [196.00, 246.94, 293.66, 392.00],
    ]
    for bar in range(int(duration / (beat * 4)) + 1):
        chord = chords[bar % 4]
        start = int(bar * beat * 4 * rate)
        end = min(total, int((bar + 1) * beat * 4 * rate))
        for n, freq in enumerate(chord):
            for i in range(start, end):
                t = (i - start) / rate
                env = min(1.0, t / 0.08) * math.exp(-t / 2.4)
                samples[i] += math.sin(2 * math.pi * freq * t) * env * 0.052
    rnd = random.Random(9)
    for b in range(int(duration / beat)):
        start = int(b * beat * rate)
        if b % 4 in (0, 2):
            length = int(rate * 0.2)
            for i in range(length):
                if start + i >= total:
                    break
                t = i / rate
                env = math.exp(-t / 0.06)
                freq = 105 - 55 * min(1.0, t / 0.2)
                samples[start + i] += math.sin(2 * math.pi * freq * t) * env * 0.42
        else:
            length = int(rate * 0.14)
            for i in range(length):
                if start + i >= total:
                    break
                t = i / rate
                env = math.exp(-t / 0.035)
                samples[start + i] += (rnd.random() * 2 - 1) * env * 0.09
        hat = start + int(beat * 0.5 * rate)
        for i in range(int(rate * 0.045)):
            if hat + i >= total:
                break
            t = i / rate
            env = math.exp(-t / 0.012)
            samples[hat + i] += (rnd.random() * 2 - 1) * env * 0.04
    motif = [440.0, 523.25, 659.25, 587.33, 523.25, 392.0, 440.0, 392.0]
    for idx, freq in enumerate(motif):
        start = int(idx * beat * 1.5 * rate) + 1000
        for i in range(int(rate * beat * 1.25)):
            if start + i >= total:
                break
            t = i / rate
            env = min(1.0, t / 0.02) * math.exp(-t / 0.45)
            samples[start + i] += math.sin(2 * math.pi * freq * t) * env * 0.06
    peak = max(abs(v) for v in samples) or 1.0
    gain = 0.82 / peak
    path = OUT / "score.wav"
    with wave.open(str(path), "wb") as fh:
        fh.setnchannels(1)
        fh.setsampwidth(2)
        fh.setframerate(rate)
        fh.writeframes(b"".join(struct.pack("<h", int(max(-1, min(1, s * gain)) * 32767)) for s in samples))
    return path


def make_clip(poster: Path, name: str) -> Path:
    out = OUT / name
    cmd = [
        FFMPEG,
        "-y",
        "-loop",
        "1",
        "-i",
        str(poster),
        "-t",
        "5",
        "-vf",
        "scale=720:1280:force_original_aspect_ratio=decrease,pad=720:1280:(ow-iw)/2:(oh-ih)/2,format=yuv420p",
        "-r",
        "30",
        "-c:v",
        "libx264",
        "-preset",
        "veryfast",
        "-crf",
        "22",
        "-movflags",
        "+faststart",
        "-an",
        str(out),
    ]
    subprocess.run(cmd, check=True, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    return out


if __name__ == "__main__":
    posters = [make_poster(i) for i in range(3)]
    score = make_score()
    print("score", score)
    for index, poster in enumerate(posters):
        clip = make_clip(poster, f"clip-{chr(97 + index)}.mp4")
        print("clip", clip)
