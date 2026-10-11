"""Regenerate icon.ico, icon.icns and icon.png from icon.svg and icon-small.svg.

    pip install resvg-py pillow
    python assets/make_icons.py

icon-small.svg is a simplified version used for 16-32 px, where the detailed
icon turns to mush. The macOS icon is drawn on Apple's icon grid (824 px body
on a 1024 px canvas, with a soft shadow); Windows and Linux get the full tile.
"""

import io
import struct
from pathlib import Path

import resvg_py
from PIL import Image, ImageFilter

HERE = Path(__file__).parent
# (svg, tile inset, tile size) in the SVG's 1024 viewBox.
LARGE = ((HERE / "icon.svg").read_text(encoding="utf-8"), 32, 960)
SMALL = ((HERE / "icon-small.svg").read_text(encoding="utf-8"), 0, 1024)
SMALL_MAX = 32


def render(svg, size):
    png = resvg_py.svg_to_bytes(svg_string=svg, width=size, height=size)
    return Image.open(io.BytesIO(bytes(png))).convert("RGBA")


def svg_for(size):
    return SMALL if size <= SMALL_MAX else LARGE


def tile(size):
    """Icon whose tile fills the canvas edge to edge (Windows, Linux)."""
    return render(svg_for(size)[0], size)


def mac(size):
    """Icon on Apple's grid: 824/1024 body, centred, with a drop shadow."""
    svg, inset, span = svg_for(size)
    body = size * 824 / 1024
    scale = body / span
    full = round(1024 * scale)
    art = render(svg, full)
    offset = round((size - body) / 2 - inset * scale)

    canvas = Image.new("RGBA", (size, size))
    if size >= 64:
        alpha = Image.new("L", (size, size))
        alpha.paste(art.getchannel("A"), (offset, offset + round(size * 10 / 1024)))
        alpha = alpha.filter(ImageFilter.GaussianBlur(size * 14 / 1024))
        shadow = Image.new("RGBA", (size, size), (0, 0, 0, 0))
        shadow.putalpha(alpha.point(lambda a: a * 0.35))
        canvas.alpha_composite(shadow)
    layer = Image.new("RGBA", (size, size))
    layer.paste(art, (offset, offset))
    canvas.alpha_composite(layer)
    return canvas


def png(im):
    buf = io.BytesIO()
    im.save(buf, format="PNG", optimize=True)
    return buf.getvalue()


def write_icns(path):
    # Every slot, including the 16/32 px @1x ones Pillow's writer leaves out.
    slots = [
        ("icp4", 16), ("icp5", 32), ("ic11", 32), ("ic12", 64), ("ic07", 128),
        ("ic13", 256), ("ic08", 256), ("ic14", 512), ("ic09", 512), ("ic10", 1024),
    ]
    cache = {}
    body = b""
    for kind, size in slots:
        data = cache.setdefault(size, png(mac(size)))
        body += kind.encode() + struct.pack(">I", 8 + len(data)) + data
    path.write_bytes(b"icns" + struct.pack(">I", 8 + len(body)) + body)


def main():
    ico_sizes = [16, 20, 24, 32, 40, 48, 64, 128, 256]
    ico = [tile(s) for s in ico_sizes]
    ico[-1].save(
        HERE / "icon.ico",
        format="ICO",
        sizes=[(s, s) for s in ico_sizes],
        append_images=ico[:-1],
    )

    write_icns(HERE / "icon.icns")

    tile(512).save(HERE / "icon.png", optimize=True)


if __name__ == "__main__":
    main()
