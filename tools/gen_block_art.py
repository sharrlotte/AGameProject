"""Generate every block sprite at final native resolution (1px minimum detail).

All sprites share a dark gunmetal palette, seamless edge-to-edge hull fill,
subtle perimeter bolts, clean matte paneling without interior stripes, and
crisp 1px dark outlines on transparent-background turrets. Triangle masks use
the same half-plane inequalities as the engine's triangle rasterizer.
"""

from pathlib import Path

from PIL import Image, ImageDraw

OUT = Path(__file__).resolve().parents[1] / "core" / "assets" / "texture" / "block"

OUTLINE = (8, 10, 14, 255)
SHADOW = (14, 16, 20, 255)
BASE_DARK = (18, 21, 26, 255)
SEAM = (24, 27, 34, 255)
BASE = (28, 32, 39, 255)
HULL_MID = (31, 35, 43, 255)
BASE_MID = (36, 40, 49, 255)
BASE_LIGHT = (44, 49, 59, 255)
EDGE_LIGHT = (54, 60, 72, 255)
STEEL = (66, 74, 88, 255)

CYAN_DIM = (38, 112, 150, 255)
CYAN = (64, 190, 230, 255)
CYAN_GLOW = (132, 228, 255, 255)
BLUE = (26, 35, 52, 255)
BLUE_LIGHT = (38, 50, 74, 255)

BRONZE = (78, 60, 32, 255)
GOLD = (118, 92, 46, 255)
AMBER = (222, 166, 58, 255)

HOT_DARK = (126, 52, 40, 255)
HOT = (206, 92, 58, 255)

CLEAR = (0, 0, 0, 0)
Color = tuple[int, int, int, int]


def clear(w: int, h: int) -> tuple[Image.Image, ImageDraw.ImageDraw]:
    """Create a transparent native-resolution canvas."""
    image = Image.new("RGBA", (w, h), CLEAR)
    return image, ImageDraw.Draw(image)


def opaque(w: int, h: int, fill: Color = BASE) -> tuple[Image.Image, ImageDraw.ImageDraw]:
    """Create an opaque native-resolution canvas filled with the shared hull base."""
    image = Image.new("RGBA", (w, h), fill)
    return image, ImageDraw.Draw(image)


def outline(image: Image.Image, color: Color = OUTLINE) -> Image.Image:
    """Add a 1px 4-connected silhouette outline around opaque pixels."""
    w, h = image.size
    out = Image.new("RGBA", (w, h), CLEAR)
    src = image.load()
    dst = out.load()
    for y in range(h):
        for x in range(w):
            if src[x, y][3] > 0:
                dst[x, y] = src[x, y]
                continue
            for dx, dy in ((-1, 0), (1, 0), (0, -1), (0, 1)):
                nx, ny = x + dx, y + dy
                if 0 <= nx < w and 0 <= ny < h and src[nx, ny][3] > 0:
                    dst[x, y] = color
                    break
    return out


def save(name: str, image: Image.Image) -> None:
    """Save a sprite PNG to the block texture directory."""
    OUT.mkdir(parents=True, exist_ok=True)
    image.save(OUT / f"{name}.png")


def px(draw: ImageDraw.ImageDraw, x: int, y: int, color: Color) -> None:
    """Draw a 1px dot."""
    draw.point((x, y), fill=color)


def bolt(draw: ImageDraw.ImageDraw, x: int, y: int, color: Color = SHADOW, sheen: Color = EDGE_LIGHT) -> None:
    """Draw a visible 3x3 machined bolt head with dark recess and upper-left sheen."""
    rect(draw, (x, y, x + 2, y + 2), color)
    rect(draw, (x, y, x + 1, y + 1), sheen)
    px(draw, x + 1, y + 1, BASE_MID)


def rect(draw: ImageDraw.ImageDraw, box: tuple[int, int, int, int], color: Color) -> None:
    """Draw an inclusive axis-aligned rectangle."""
    draw.rectangle(box, fill=color)


def hline(draw: ImageDraw.ImageDraw, x0: int, x1: int, y: int, color: Color) -> None:
    """Draw a 1px horizontal line across inclusive x bounds."""
    draw.rectangle((x0, y, x1, y), fill=color)


def vline(draw: ImageDraw.ImageDraw, x: int, y0: int, y1: int, color: Color) -> None:
    """Draw a 1px vertical line across inclusive y bounds."""
    draw.rectangle((x, y0, x, y1), fill=color)


def chamfer_box(
    draw: ImageDraw.ImageDraw,
    box: tuple[int, int, int, int],
    chamfer: int,
    color: Color,
) -> None:
    """Draw a symmetric octagonal box with 45-degree corner chamfers."""
    x0, y0, x1, y1 = box
    if chamfer <= 0:
        draw.rectangle(box, fill=color)
        return
    draw.rectangle((x0, y0 + chamfer, x1, y1 - chamfer), fill=color)
    for i in range(chamfer):
        inset = chamfer - i
        hline(draw, x0 + inset, x1 - inset, y0 + i, color)
        hline(draw, x0 + inset, x1 - inset, y1 - i, color)


def bevel_rect(
    draw: ImageDraw.ImageDraw,
    box: tuple[int, int, int, int],
    fill: Color,
    highlight: Color = EDGE_LIGHT,
    shadow: Color = SHADOW,
) -> None:
    """Draw a metallic interior plate with 1px upper-left highlight and lower-right shadow."""
    x0, y0, x1, y1 = box
    rect(draw, box, fill)
    hline(draw, x0, x1 - 1, y0, highlight)
    vline(draw, x0, y0, y1 - 1, highlight)
    hline(draw, x0 + 1, x1, y1, shadow)
    vline(draw, x1, y0 + 1, y1, shadow)


def edge_bolts(
    draw: ImageDraw.ImageDraw,
    box: tuple[int, int, int, int],
    step: int = 18,
    color: Color = SEAM,
    sheen: Color = BASE_LIGHT,
) -> None:
    """Place subtle 1px bolts evenly around the perimeter of box."""
    x0, y0, x1, y1 = box
    for x in range(x0, x1 + 1, step):
        bolt(draw, x, y0, color, sheen)
        bolt(draw, x, y1, color, sheen)
    for y in range(y0 + step, y1, step):
        bolt(draw, x0, y, color, sheen)
        bolt(draw, x1, y, color, sheen)
    bolt(draw, x1, y0, color, sheen)
    bolt(draw, x1, y1, color, sheen)


def draw_hull() -> Image.Image:
    """Generate seamless matte hull plating with perimeter bolts and no stripes."""
    image, d = opaque(64, 64)
    rect(d, (4, 4, 59, 59), HULL_MID)
    edge_bolts(d, (7, 7, 54, 54), step=23, color=SHADOW, sheen=BASE_LIGHT)
    return image


def draw_steel() -> Image.Image:
    """Generate bolted heavy steel armor plating with seamless edges."""
    image, d = opaque(64, 64)
    bevel_rect(d, (5, 5, 58, 58), BASE_MID, BASE_LIGHT, SEAM)
    bevel_rect(d, (16, 16, 47, 47), BASE, BASE_LIGHT, SHADOW)
    edge_bolts(d, (8, 8, 54, 54), step=15, color=SHADOW, sheen=EDGE_LIGHT)
    return image


def draw_scrap() -> Image.Image:
    """Generate X-braced industrial scrap armor with perimeter bolts."""
    image, d = opaque(64, 64)
    rect(d, (6, 6, 57, 57), BASE_DARK)
    d.line([(8, 8), (55, 55)], fill=BASE_MID, width=5)
    d.line([(8, 55), (55, 8)], fill=BASE_MID, width=5)
    d.line([(8, 8), (55, 55)], fill=BASE_LIGHT, width=1)
    d.line([(8, 55), (55, 8)], fill=BASE_LIGHT, width=1)
    bevel_rect(d, (23, 23, 40, 40), BASE_MID, BASE_LIGHT, SHADOW)
    edge_bolts(d, (8, 8, 53, 53), step=22, color=SEAM, sheen=BASE_MID)
    edge_bolts(d, (25, 25, 37, 37), step=12, color=SHADOW, sheen=EDGE_LIGHT)
    return image


def draw_turret_base() -> Image.Image:
    """Generate a seamless bolted hull socket for mounting rotating turrets."""
    image, d = opaque(64, 64)
    chamfer_box(d, (10, 10, 53, 53), 12, BASE_DARK)
    chamfer_box(d, (16, 16, 47, 47), 9, BASE_MID)
    d.ellipse((22, 22, 41, 41), fill=SHADOW)
    d.ellipse((26, 26, 37, 37), fill=BASE_DARK)
    edge_bolts(d, (7, 7, 54, 54), step=23, color=SEAM, sheen=BASE_LIGHT)
    return image


def draw_nucleus() -> Image.Image:
    """Generate the central power core with seamless outer hull skirt and bolts."""
    image, d = opaque(160, 160)
    chamfer_box(d, (12, 12, 147, 147), 20, SEAM)
    chamfer_box(d, (14, 14, 145, 145), 19, BASE_MID)
    chamfer_box(d, (26, 26, 133, 133), 15, BASE_DARK)

    rect(d, (77, 26, 82, 133), BRONZE)
    rect(d, (78, 26, 81, 133), GOLD)
    rect(d, (26, 77, 133, 82), BRONZE)
    rect(d, (26, 78, 133, 81), GOLD)

    d.ellipse((48, 48, 111, 111), fill=CYAN_DIM)
    d.ellipse((56, 56, 103, 103), fill=CYAN)
    d.ellipse((67, 67, 92, 92), fill=CYAN_GLOW)

    edge_bolts(d, (10, 10, 147, 147), step=34, color=SEAM, sheen=BASE_LIGHT)
    for x, y in ((22, 22), (136, 22), (22, 136), (136, 136)):
        bolt(d, x, y, SHADOW, GOLD)
    return image


def draw_battery() -> Image.Image:
    """Generate a blue-framed energy storage block with cyan charge cells and edge bolts."""
    image, d = opaque(64, 64)
    bevel_rect(d, (6, 6, 57, 57), BLUE, BLUE_LIGHT, SHADOW)
    rect(d, (14, 14, 49, 49), BASE_DARK)
    for y in (18, 28, 38):
        rect(d, (18, y, 45, y + 6), CYAN_DIM)
        rect(d, (20, y + 1, 43, y + 5), CYAN)
        hline(d, 20, 32, y + 1, CYAN_GLOW)
    edge_bolts(d, (8, 8, 53, 53), step=22, color=SEAM, sheen=BASE_LIGHT)
    return image


def draw_booster() -> Image.Image:
    """Generate an overdrive emitter block with upward cyan arcs and edge bolts."""
    image, d = opaque(64, 64)
    chamfer_box(d, (8, 8, 55, 55), 8, BASE_DARK)
    d.arc((14, 14, 49, 49), 210, 330, fill=CYAN_DIM, width=3)
    d.arc((21, 21, 42, 42), 210, 330, fill=CYAN, width=2)
    chamfer_box(d, (27, 33, 36, 44), 2, BASE_MID)
    rect(d, (30, 36, 33, 39), CYAN_GLOW)
    edge_bolts(d, (7, 7, 54, 54), step=23, color=SEAM, sheen=BASE_LIGHT)
    return image


def draw_thruster() -> Image.Image:
    """Generate an ion thruster with seamless upper/side hull, edge bolts, and nozzle."""
    image, d = opaque(64, 64)
    d.polygon([(16, 10), (47, 10), (55, 52), (8, 52)], fill=BASE_DARK)
    d.polygon([(22, 18), (41, 18), (47, 46), (16, 46)], fill=CYAN_DIM)
    d.polygon([(26, 24), (37, 24), (41, 42), (22, 42)], fill=CYAN)
    rect(d, (30, 32, 33, 36), CYAN_GLOW)
    bevel_rect(d, (7, 50, 56, 56), BASE_LIGHT, EDGE_LIGHT, SHADOW)
    rect(d, (9, 57, 54, 63), BASE_DARK)
    for x in (7, 30, 54):
        bolt(d, x, 6, SEAM, BASE_LIGHT)
    for y in (22, 40):
        bolt(d, 5, y, SEAM, BASE_LIGHT)
        bolt(d, 56, y, SEAM, BASE_LIGHT)
    return image


def triangle_mask(width: int, height: int, base: str) -> Image.Image:
    """Compute the exact alpha mask matching the Rust triangle hitbox rasterizer."""
    mask = Image.new("L", (width, height), 0)
    pixels = []
    for row in range(height):
        for col in range(width):
            px_coord = col + 0.5 - width / 2
            py_coord = height / 2 - (row + 0.5)
            if base == "right":
                inside = px_coord / width + py_coord / height <= 0.0
            else:
                inside = (
                    px_coord / width >= py_coord / (2 * height) - 0.25
                    and px_coord / width <= 0.25 - py_coord / (2 * height)
                )
            pixels.append(255 if inside else 0)
    mask.putdata(pixels)
    return mask


def draw_triangle(width: int, height: int, base: str) -> Image.Image:
    """Generate a clean bolted hull triangle without interior stripes."""
    image, d = opaque(width, height)
    rect(d, (4, 4, width - 5, height - 5), HULL_MID)
    if base == "right":
        for x in range(8, width - 12, 22):
            bolt(d, x, height - 10, SHADOW, BASE_LIGHT)
        for y in range(12, height - 12, 22):
            bolt(d, 7, y, SHADOW, BASE_LIGHT)
    else:
        for x in range(10, width - 10, 22):
            bolt(d, x, height - 10, SHADOW, BASE_LIGHT)
    image.putalpha(triangle_mask(width, height, base))
    return image


def draw_lancer() -> Image.Image:
    """Generate the twin-rail laser turret with 1px dark outline."""
    image, d = clear(64, 64)
    for bx in (17, 38):
        bevel_rect(d, (bx, 4, bx + 8, 42), BASE_LIGHT, STEEL, BASE)
        rect(d, (bx + 2, 8, bx + 6, 38), BASE_DARK)
        rect(d, (bx + 3, 10, bx + 5, 36), CYAN)
        vline(d, bx + 4, 10, 18, CYAN_GLOW)
    chamfer_box(d, (12, 40, 51, 58), 5, BASE_LIGHT)
    hline(d, 17, 46, 40, EDGE_LIGHT)
    rect(d, (18, 44, 45, 54), BASE_MID)
    rect(d, (26, 46, 37, 52), BASE_DARK)
    rect(d, (28, 47, 35, 51), GOLD)
    edge_bolts(d, (17, 44, 45, 53), step=28, color=SHADOW, sheen=STEEL)
    return outline(image)


def draw_colaris() -> Image.Image:
    """Generate the compact twin-cannon turret with 1px dark outline."""
    image, d = clear(64, 64)
    for bx in (17, 38):
        bevel_rect(d, (bx, 6, bx + 8, 24), BASE_LIGHT, STEEL, BASE)
        rect(d, (bx + 2, 10, bx + 6, 20), BASE_DARK)
    chamfer_box(d, (12, 20, 51, 54), 6, BASE_LIGHT)
    hline(d, 18, 45, 20, EDGE_LIGHT)
    rect(d, (16, 26, 22, 48), BASE_MID)
    rect(d, (41, 26, 47, 48), BASE_MID)
    rect(d, (24, 28, 39, 46), BASE_DARK)
    rect(d, (26, 30, 37, 42), CYAN_DIM)
    rect(d, (28, 32, 35, 40), CYAN)
    px(d, 29, 33, CYAN_GLOW)
    edge_bolts(d, (16, 25, 46, 48), step=30, color=SHADOW, sheen=STEEL)
    return outline(image)


def draw_swarmer() -> Image.Image:
    """Generate the 4-cell missile pod turret with 1px dark outline."""
    image, d = clear(64, 64)
    chamfer_box(d, (8, 8, 55, 55), 5, BASE_LIGHT)
    hline(d, 13, 50, 8, STEEL)
    hline(d, 13, 50, 55, SHADOW)
    for cx, cy in ((16, 16), (38, 16), (16, 38), (38, 38)):
        rect(d, (cx, cy, cx + 9, cy + 9), BASE_DARK)
        d.ellipse((cx + 2, cy + 2, cx + 7, cy + 7), fill=HOT_DARK)
        d.ellipse((cx + 3, cy + 3, cx + 6, cy + 6), fill=HOT)
        px(d, cx + 4, cy + 3, AMBER)
    edge_bolts(d, (12, 12, 50, 50), step=19, color=SHADOW, sheen=STEEL)
    return outline(image)


def draw_spectre() -> Image.Image:
    """Generate the heavy twin autocannon turret with 1px dark outline."""
    image, d = clear(128, 128)
    for bx in (28, 79):
        bevel_rect(d, (bx, 8, bx + 20, 84), BASE_LIGHT, STEEL, BASE)
        rect(d, (bx, 8, bx + 20, 14), STEEL)
        rect(d, (bx + 4, 18, bx + 16, 78), BASE_DARK)
        rect(d, (bx + 8, 20, bx + 12, 76), BRONZE)
        rect(d, (bx + 9, 20, bx + 11, 76), GOLD)
    chamfer_box(d, (16, 80, 111, 118), 8, BASE_LIGHT)
    hline(d, 24, 103, 80, EDGE_LIGHT)
    hline(d, 24, 103, 118, SHADOW)
    bevel_rect(d, (22, 86, 38, 110), BASE_MID, BASE_LIGHT, SHADOW)
    bevel_rect(d, (89, 86, 105, 110), BASE_MID, BASE_LIGHT, SHADOW)
    rect(d, (50, 88, 77, 110), BASE_DARK)
    d.ellipse((58, 93, 69, 104), fill=CYAN_DIM)
    d.ellipse((60, 95, 67, 102), fill=CYAN)
    px(d, 62, 97, CYAN_GLOW)
    edge_bolts(d, (24, 86, 102, 111), step=26, color=SHADOW, sheen=STEEL)
    return outline(image)


def draw_meltdown() -> Image.Image:
    """Generate the heavy thermal beam turret with 1px dark outline."""
    image, d = clear(128, 128)
    for bx in (24, 79):
        bevel_rect(d, (bx, 8, bx + 24, 80), BASE_LIGHT, STEEL, BASE)
        hline(d, bx, bx + 24, 8, STEEL)
        rect(d, (bx + 6, 14, bx + 18, 74), HOT_DARK)
        rect(d, (bx + 10, 16, bx + 14, 72), HOT)
        rect(d, (bx + 11, 16, bx + 13, 30), AMBER)
    chamfer_box(d, (16, 76, 111, 118), 8, BASE_LIGHT)
    hline(d, 24, 103, 76, EDGE_LIGHT)
    hline(d, 24, 103, 118, SHADOW)
    bevel_rect(d, (22, 82, 38, 110), BASE_MID, BASE_LIGHT, SHADOW)
    bevel_rect(d, (89, 82, 105, 110), BASE_MID, BASE_LIGHT, SHADOW)
    rect(d, (48, 84, 79, 110), BASE_DARK)
    rect(d, (52, 88, 75, 106), HOT_DARK)
    rect(d, (56, 92, 71, 102), HOT)
    rect(d, (60, 94, 67, 98), AMBER)
    edge_bolts(d, (24, 83, 102, 111), step=26, color=SHADOW, sheen=STEEL)
    return outline(image)


def draw_foreshadow() -> Image.Image:
    """Generate the 1x4 railgun turret (64x256) with 1px dark outline."""
    image, d = clear(64, 256)
    for bx, inner in ((13, 21), (42, 42)):
        bevel_rect(d, (bx, 4, bx + 8, 140), BASE_LIGHT, STEEL, BASE)
        vline(d, inner, 6, 136, STEEL)
    for y in range(14, 132, 14):
        rect(d, (22, y, 41, y + 6), CYAN_DIM)
        rect(d, (24, y + 1, 39, y + 4), CYAN)
        hline(d, 26, 37, y + 1, CYAN_GLOW)
    chamfer_box(d, (8, 136, 55, 250), 8, BASE_LIGHT)
    hline(d, 16, 47, 136, EDGE_LIGHT)
    hline(d, 16, 47, 250, SHADOW)
    rect(d, (12, 148, 51, 242), BASE_MID)
    rect(d, (18, 168, 45, 224), BASE_DARK)
    rect(d, (22, 172, 41, 220), BRONZE)
    rect(d, (24, 174, 39, 218), GOLD)
    rect(d, (28, 180, 35, 208), AMBER)
    edge_bolts(d, (14, 150, 48, 240), step=17, color=SHADOW, sheen=STEEL)
    return outline(image)


def main() -> None:
    """Regenerate every block sprite PNG and print output dimensions."""
    save("turret-base", draw_turret_base())
    save("scrap", draw_scrap())
    save("steel", draw_steel())
    save("nucleus", draw_nucleus())
    save("booster", draw_booster())
    save("hull", draw_hull())
    save("thruster", draw_thruster())
    save("battery", draw_battery())

    save("wedge-1x1", draw_triangle(64, 64, "right"))
    save("cap-1x1", draw_triangle(64, 64, "isosceles"))
    save("plate-2x2", draw_triangle(128, 128, "right"))
    save("fin-1x2", draw_triangle(64, 128, "isosceles"))
    save("spike-1x3", draw_triangle(64, 192, "isosceles"))

    save("lancer", draw_lancer())
    save("colaris", draw_colaris())
    save("swarmer", draw_swarmer())
    save("spectre", draw_spectre())
    save("meltdown", draw_meltdown())
    save("foreshadow", draw_foreshadow())

    for path in sorted(OUT.glob("*.png")):
        with Image.open(path) as image:
            print(f"{path.name}: {image.width}x{image.height}")


if __name__ == "__main__":
    main()
