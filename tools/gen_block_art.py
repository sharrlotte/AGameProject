"""Generate every block sprite at final native resolution (1px minimum detail).

All sprites share a dark gunmetal palette, seamless edge-to-edge hull fill,
corner-prioritized perimeter bolts on structural pieces, clean matte paneling,
and crisp 1px dark gunmetal framing on transparent-background turrets.
Triangle masks use the same half-plane inequalities as the engine's triangle rasterizer.

Note: hull.png is intentionally preserved and ignored during batch regeneration
as the approved baseline reference standard.
"""

import math
from pathlib import Path

from PIL import Image, ImageDraw

OUT = Path(__file__).resolve().parents[1] / "core" / "assets" / "texture" / "block"

# Grounded military/industrial sci-fi hardware palette
OUTLINE = (14, 16, 20, 255)
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


def bolt(draw: ImageDraw.ImageDraw, x: int, y: int, color: Color = SHADOW, sheen: Color = BASE_LIGHT) -> None:
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
    """Place subtle 1px bolts evenly around the perimeter of box, prioritizing corners."""
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
    """Generate heavy composite ballistic steel armor plating with seamless edges."""
    image, d = opaque(64, 64, fill=BASE)
    rect(d, (4, 4, 59, 59), BASE_MID)
    hline(d, 4, 58, 4, EDGE_LIGHT)
    vline(d, 4, 4, 58, EDGE_LIGHT)
    hline(d, 5, 59, 59, SHADOW)
    vline(d, 59, 5, 59, SHADOW)

    for cx, cy in ((4, 4), (55, 4), (4, 55), (55, 55)):
        rect(d, (cx, cy, cx + 4, cy + 4), BASE_DARK)

    chamfer_box(d, (12, 12, 51, 51), 6, BASE_LIGHT)
    hline(d, 18, 45, 12, STEEL)
    vline(d, 12, 18, 45, STEEL)
    hline(d, 18, 45, 51, SHADOW)
    vline(d, 51, 18, 45, SHADOW)

    rect(d, (18, 18, 45, 45), BASE)
    hline(d, 18, 44, 18, SHADOW)
    vline(d, 18, 18, 44, SHADOW)
    hline(d, 19, 45, 45, BASE_MID)
    vline(d, 45, 19, 45, BASE_MID)

    bevel_rect(d, (27, 27, 36, 36), BASE_MID, EDGE_LIGHT, SHADOW)
    rect(d, (29, 29, 34, 34), BASE_DARK)
    px(d, 31, 31, BASE_LIGHT)
    px(d, 32, 31, BASE_LIGHT)
    px(d, 31, 32, BASE_MID)
    px(d, 32, 32, SHADOW)

    for bx, by in ((7, 7), (54, 7), (7, 54), (54, 54)):
        bolt(d, bx, by, SHADOW, STEEL)
    for bx, by in ((30, 7), (7, 30), (54, 30), (30, 54)):
        bolt(d, bx, by, SHADOW, EDGE_LIGHT)
    return image


def draw_scrap() -> Image.Image:
    """Generate salvaged multi-plate armor with reinforced weld seams and heavy bolts."""
    image, d = opaque(64, 64, fill=BASE)
    rect(d, (4, 4, 59, 59), BASE_DARK)

    # Upper-left salvaged plate
    bevel_rect(d, (4, 4, 39, 31), BASE_MID, EDGE_LIGHT, SHADOW)
    rect(d, (6, 6, 37, 29), BASE)

    # Lower-right heavy patch plate
    bevel_rect(d, (25, 27, 59, 59), BASE_LIGHT, STEEL, SHADOW)
    rect(d, (27, 29, 57, 57), BASE_MID)

    # Industrial weld bead across overlapping plates
    for i in range(16):
        wx = 20 + i * 2
        wy = 36 - i
        d.point((wx, wy), fill=STEEL)
        d.point((wx + 1, wy), fill=BASE_DARK)
        d.point((wx, wy + 1), fill=BRONZE)

    # Lower-left reinforcement strap
    bevel_rect(d, (4, 32, 24, 59), BASE_MID, EDGE_LIGHT, SHADOW)

    # Heavy improvised hex bolts
    for bx, by in ((7, 7), (34, 7), (7, 54), (54, 54), (54, 30), (28, 54)):
        bolt(d, bx, by, SHADOW, STEEL)
    bolt(d, 22, 24, SHADOW, BRONZE)
    bolt(d, 40, 42, SHADOW, BRONZE)
    return image


def draw_turret_base() -> Image.Image:
    """Generate a motorized traverse turret ring socket with azimuth gear teeth."""
    image, d = opaque(64, 64, fill=BASE)
    chamfer_box(d, (5, 5, 58, 58), 10, BASE_MID)
    hline(d, 15, 48, 5, EDGE_LIGHT)
    vline(d, 5, 15, 48, EDGE_LIGHT)
    hline(d, 15, 48, 58, SHADOW)
    vline(d, 58, 15, 48, SHADOW)

    chamfer_box(d, (11, 11, 52, 52), 12, BASE_DARK)

    d.ellipse((14, 14, 49, 49), fill=BASE_LIGHT)
    d.ellipse((15, 15, 48, 48), fill=BASE)

    # Geared azimuth ring teeth
    for angle_deg in range(0, 360, 24):
        rad = math.radians(angle_deg)
        tx1 = int(round(31.5 + 15.0 * math.cos(rad)))
        ty1 = int(round(31.5 + 15.0 * math.sin(rad)))
        tx2 = int(round(31.5 + 17.0 * math.cos(rad)))
        ty2 = int(round(31.5 + 17.0 * math.sin(rad)))
        d.line([(tx1, ty1), (tx2, ty2)], fill=STEEL, width=1)

    d.ellipse((20, 20, 43, 43), fill=BASE_DARK)
    d.ellipse((22, 22, 41, 41), fill=SHADOW)

    d.ellipse((27, 27, 36, 36), fill=BASE_MID)
    d.ellipse((29, 29, 34, 34), fill=BASE_DARK)
    px(d, 31, 31, STEEL)

    rect(d, (20, 30, 23, 33), BRONZE)
    rect(d, (40, 30, 43, 33), BRONZE)

    edge_bolts(d, (7, 7, 54, 54), step=23, color=SHADOW, sheen=BASE_LIGHT)
    return image


def draw_nucleus() -> Image.Image:
    """Generate a heavy 160x160 nuclear/fusion containment reactor."""
    image, d = opaque(160, 160, fill=BASE)

    chamfer_box(d, (4, 4, 155, 155), 24, BASE_MID)
    hline(d, 28, 131, 4, STEEL)
    vline(d, 4, 28, 131, STEEL)
    hline(d, 28, 131, 155, SHADOW)
    vline(d, 155, 28, 131, SHADOW)

    # 4 Corner Heat Radiator Fin Banks
    for rx, ry in ((12, 12), (116, 12), (12, 116), (116, 116)):
        rect(d, (rx, ry, rx + 31, ry + 31), BASE_DARK)
        for i in range(2, 30, 4):
            hline(d, rx + 2, rx + 29, ry + i, BASE_LIGHT)
            hline(d, rx + 2, rx + 29, ry + i + 1, SHADOW)

    # Primary Coolant Circulation Manifolds
    bevel_rect(d, (12, 72, 147, 87), BASE_LIGHT, STEEL, SHADOW)
    rect(d, (14, 75, 145, 84), BASE_DARK)
    hline(d, 14, 145, 78, BRONZE)
    hline(d, 14, 145, 81, GOLD)

    bevel_rect(d, (72, 12, 87, 147), BASE_LIGHT, STEEL, SHADOW)
    rect(d, (75, 14, 84, 145), BASE_DARK)
    vline(d, 78, 14, 145, BRONZE)
    vline(d, 81, 14, 145, GOLD)

    # Magnetic Containment Torus
    d.ellipse((34, 34, 125, 125), fill=BASE_LIGHT)
    d.ellipse((36, 36, 123, 123), fill=BASE_MID)
    d.ellipse((42, 42, 117, 117), fill=BASE_DARK)

    for a in range(0, 360, 22):
        rad = math.radians(a)
        cx1 = int(round(79.5 + 38.0 * math.cos(rad)))
        cy1 = int(round(79.5 + 38.0 * math.sin(rad)))
        cx2 = int(round(79.5 + 46.0 * math.cos(rad)))
        cy2 = int(round(79.5 + 46.0 * math.sin(rad)))
        d.line([(cx1, cy1), (cx2, cy2)], fill=GOLD, width=2)

    d.ellipse((50, 50, 109, 109), fill=BASE_DARK)
    d.ellipse((54, 54, 105, 105), fill=SHADOW)

    # Deep Plasma Core Glow
    d.ellipse((58, 58, 101, 101), fill=CYAN_DIM)
    d.ellipse((66, 66, 93, 93), fill=CYAN)
    d.ellipse((73, 73, 86, 86), fill=CYAN_GLOW)

    for px_coord, py_coord in ((79, 52), (79, 107), (52, 79), (107, 79)):
        rect(d, (px_coord - 2, py_coord - 2, px_coord + 2, py_coord + 2), STEEL)

    edge_bolts(d, (8, 8, 151, 151), step=35, color=SHADOW, sheen=STEEL)
    for bx, by in ((22, 22), (135, 22), (22, 135), (135, 135)):
        bolt(d, bx, by, SHADOW, GOLD)
    return image


def draw_battery() -> Image.Image:
    """Generate a titanium-framed industrial capacitor storage bank."""
    image, d = opaque(64, 64, fill=BASE)
    bevel_rect(d, (4, 4, 59, 59), BLUE, BLUE_LIGHT, SHADOW)
    rect(d, (9, 9, 54, 54), BASE_DARK)

    for cy in (12, 26, 40):
        bevel_rect(d, (12, cy, 51, cy + 10), BASE_MID, EDGE_LIGHT, SHADOW)
        rect(d, (15, cy + 2, 48, cy + 8), BASE_DARK)

        rect(d, (15, cy + 3, 17, cy + 7), GOLD)
        rect(d, (46, cy + 3, 48, cy + 7), GOLD)

        rect(d, (19, cy + 3, 44, cy + 7), BLUE)
        hline(d, 20, 43, cy + 3, BLUE_LIGHT)
        hline(d, 20, 43, cy + 5, CYAN_DIM)
        hline(d, 22, 36, cy + 5, CYAN)
        px(d, 23, cy + 5, CYAN_GLOW)

        for vx in (26, 31, 36, 41):
            vline(d, vx, cy + 2, cy + 8, BASE_DARK)

    vline(d, 10, 10, 53, BRONZE)
    vline(d, 53, 10, 53, BRONZE)

    edge_bolts(d, (7, 7, 54, 54), step=23, color=SHADOW, sheen=BASE_LIGHT)
    return image


def draw_booster() -> Image.Image:
    """Generate an industrial overdrive impulse module with induction stator coils."""
    image, d = opaque(64, 64, fill=BASE)
    chamfer_box(d, (5, 5, 58, 58), 8, BASE_MID)
    hline(d, 13, 50, 5, EDGE_LIGHT)
    vline(d, 5, 13, 50, EDGE_LIGHT)
    hline(d, 13, 50, 58, SHADOW)
    vline(d, 58, 13, 50, SHADOW)

    chamfer_box(d, (10, 10, 53, 53), 10, BASE_DARK)

    d.ellipse((14, 14, 49, 49), fill=BASE_LIGHT)
    d.ellipse((16, 16, 47, 47), fill=BASE)

    for r in (22, 20, 18):
        d.ellipse((31 - r, 31 - r, 32 + r, 32 + r), outline=BRONZE, width=1)
    d.ellipse((20, 20, 43, 43), fill=BASE_DARK)

    rect(d, (29, 13, 34, 21), BASE_LIGHT)
    rect(d, (29, 42, 34, 50), BASE_LIGHT)
    rect(d, (13, 29, 21, 34), BASE_LIGHT)
    rect(d, (42, 29, 50, 34), BASE_LIGHT)

    d.ellipse((25, 25, 38, 38), fill=SHADOW)
    d.ellipse((27, 27, 36, 36), fill=CYAN_DIM)
    d.ellipse((29, 29, 34, 34), fill=CYAN)
    px(d, 31, 31, CYAN_GLOW)

    edge_bolts(d, (7, 7, 54, 54), step=23, color=SHADOW, sheen=BASE_LIGHT)
    return image


def draw_thruster() -> Image.Image:
    """Generate an ion propulsion thruster with gimbal actuators and cooling strakes."""
    image, d = opaque(64, 64, fill=BASE)
    rect(d, (4, 4, 59, 24), BASE_MID)
    hline(d, 4, 59, 4, EDGE_LIGHT)
    vline(d, 4, 4, 24, EDGE_LIGHT)
    vline(d, 59, 4, 24, SHADOW)

    bevel_rect(d, (16, 8, 47, 20), BASE_LIGHT, STEEL, SHADOW)
    for fx in (20, 28, 36, 44):
        rect(d, (fx - 1, 6, fx + 1, 9), BRONZE)
        rect(d, (fx, 7, fx, 8), GOLD)

    bevel_rect(d, (8, 16, 13, 34), BASE_LIGHT, STEEL, SHADOW)
    bevel_rect(d, (50, 16, 55, 34), BASE_LIGHT, STEEL, SHADOW)
    vline(d, 10, 20, 30, BASE_DARK)
    vline(d, 52, 20, 30, BASE_DARK)

    nozzle_pts = [(20, 22), (43, 22), (54, 56), (9, 56)]
    d.polygon(nozzle_pts, fill=BASE_DARK)

    inner_nozzle = [(22, 24), (41, 24), (51, 54), (12, 54)]
    d.polygon(inner_nozzle, fill=BASE_MID)

    d.line([(22, 24), (13, 54)], fill=STEEL, width=2)
    d.line([(24, 24), (17, 54)], fill=BASE_LIGHT, width=2)
    d.line([(39, 24), (48, 54)], fill=SEAM, width=2)
    d.line([(41, 24), (51, 54)], fill=SHADOW, width=2)

    for tx in (22, 27, 32, 37, 42):
        bx_bot = int(round(31.5 + (tx - 31.5) * 2.2))
        d.line([(tx, 26), (bx_bot, 52)], fill=BASE_DARK, width=1)
        d.line([(tx - 1, 26), (bx_bot - 1, 52)], fill=EDGE_LIGHT, width=1)

    d.polygon([(26, 24), (37, 24), (39, 30), (24, 30)], fill=BASE_DARK)
    d.polygon([(28, 25), (35, 25), (36, 29), (27, 29)], fill=CYAN_DIM)
    rect(d, (30, 26, 33, 28), CYAN)
    px(d, 31, 27, CYAN_GLOW)

    bevel_rect(d, (8, 55, 55, 58), BASE_LIGHT, STEEL, SHADOW)
    rect(d, (10, 59, 53, 63), BASE_DARK)

    for bx in (7, 30, 54):
        bolt(d, bx, 6, SHADOW, BASE_LIGHT)
    for by in (16, 26):
        bolt(d, 5, by, SHADOW, BASE_LIGHT)
        bolt(d, 56, by, SHADOW, BASE_LIGHT)
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


def draw_right_triangle(width: int, height: int) -> Image.Image:
    """Generate a clean bolted right triangle hull with corner-prioritized perimeter bolts."""
    image = Image.new("RGBA", (width, height), BASE)
    d = ImageDraw.Draw(image)
    scale = width // 64
    diag_offset = 7 if scale == 1 else 9
    rim = 4
    for y in range(rim, height - rim):
        for x in range(rim, width - rim):
            if y - x >= diag_offset:
                d.point((x, y), fill=HULL_MID)

    if scale == 1:
        bolts = [
            (7, 54),   # Bottom-left 90-degree corner
            (7, 18),   # Top 45-degree corner
            (43, 54),  # Bottom-right 45-degree corner
            (25, 36),  # Hypotenuse midpoint
        ]
    else:
        bolts = [
            (7, 118), (7, 26), (99, 118),
            (7, 49), (7, 72), (7, 95),
            (30, 118), (53, 118), (76, 118),
            (25, 44), (43, 62), (61, 80), (80, 99),
        ]
    for bx, by in bolts:
        bolt(d, bx, by, SHADOW, BASE_LIGHT)
    image.putalpha(triangle_mask(width, height, "right"))
    return image


def draw_isosceles_triangle(width: int, height: int) -> Image.Image:
    """Generate a clean bolted isosceles triangle with corner-prioritized perimeter bolts."""
    image = Image.new("RGBA", (width, height), BASE)
    d = ImageDraw.Draw(image)

    apex_x = (width - 1) / 2.0
    apex_y = 0.5
    bl_x = -0.5
    bl_y = height - 0.5
    dy = bl_y - apex_y
    dx = bl_x - apex_x
    denom = math.hypot(dy, dx)

    for y in range(4, height - 4):
        for x in range(width):
            d_left = (dy * (x - apex_x) - dx * (y - apex_y)) / denom
            d_right = (dy * ((width - 1 - x) - apex_x) - dx * (y - apex_y)) / denom
            d_bot = (height - 0.5) - y
            if d_left >= 4.0 and d_right >= 4.0 and d_bot >= 4.0:
                d.point((x, y), fill=HULL_MID)

    # 1. Apex corner bolt
    apex_bolt_y = int(round(8.0 * denom / (width / 2.0) + 0.5))
    apex_bolt_x = int(round(apex_x - 1))

    # 2. Bottom-left & bottom-right corner bolts
    bl_by = height - 10
    bl_bx = int(round((8.0 * denom + dx * (bl_by + 1 - apex_y)) / dy + apex_x - 1))
    br_bx = width - 3 - bl_bx

    bolts = [(bl_bx, bl_by), (br_bx, bl_by), (apex_bolt_x, apex_bolt_y)]

    # 3. Bottom edge midpoint
    if (br_bx - bl_bx) >= 28:
        mid_bx = int(round(apex_x - 1))
        bolts.append((mid_bx, bl_by))

    # 4. Diagonal edge bolts between bottom corners and apex
    diag_steps = max(1, int(round(math.hypot(apex_bolt_x - bl_bx, bl_by - apex_bolt_y) / 24.0)))
    for s in range(1, diag_steps):
        t = s / diag_steps
        left_x = int(round(bl_bx * (1 - t) + apex_bolt_x * t))
        left_y = int(round(bl_by * (1 - t) + apex_bolt_y * t))
        right_x = width - 3 - left_x
        bolts.append((left_x, left_y))
        bolts.append((right_x, left_y))

    for bx, by in bolts:
        bolt(d, bx, by, SHADOW, BASE_LIGHT)

    image.putalpha(triangle_mask(width, height, "isosceles"))
    return image


def draw_colaris() -> Image.Image:
    """Generate the realistic twin autocannon turret with 1px dark gunmetal outline."""
    image, d = clear(64, 64)

    # Traverse ring / turntable
    d.ellipse((19, 43, 44, 57), fill=BASE_DARK)
    d.ellipse((20, 44, 43, 56), fill=BASE_MID)
    d.ellipse((22, 45, 41, 54), fill=SHADOW)
    d.ellipse((23, 46, 40, 53), fill=BASE)

    # Side ammo drums
    chamfer_box(d, (11, 33, 21, 49), 3, BASE_MID)
    hline(d, 13, 20, 33, EDGE_LIGHT)
    vline(d, 11, 35, 47, EDGE_LIGHT)
    hline(d, 13, 20, 49, SHADOW)
    vline(d, 21, 35, 47, SHADOW)
    for ry in (37, 41, 45):
        hline(d, 13, 19, ry, SEAM)
        hline(d, 13, 19, ry - 1, BASE_LIGHT)

    chamfer_box(d, (42, 33, 52, 49), 3, BASE_MID)
    hline(d, 44, 51, 33, EDGE_LIGHT)
    vline(d, 42, 35, 47, BASE_LIGHT)
    hline(d, 44, 51, 49, SHADOW)
    vline(d, 52, 35, 47, SHADOW)
    for ry in (37, 41, 45):
        hline(d, 44, 50, ry, SEAM)
        hline(d, 44, 50, ry - 1, BASE_LIGHT)

    # Main armored turret body
    chamfer_box(d, (19, 26, 44, 51), 4, BASE_LIGHT)
    hline(d, 22, 41, 26, STEEL)
    vline(d, 19, 29, 48, EDGE_LIGHT)
    hline(d, 22, 41, 51, SHADOW)
    vline(d, 44, 29, 48, SHADOW)

    rect(d, (23, 29, 40, 46), BASE)
    hline(d, 23, 40, 29, BASE_LIGHT)
    vline(d, 23, 30, 45, BASE_LIGHT)
    hline(d, 24, 40, 46, SHADOW)
    vline(d, 40, 30, 45, SHADOW)

    rect(d, (29, 34, 34, 41), BASE_DARK)
    rect(d, (30, 35, 33, 39), CYAN_DIM)
    px(d, 31, 36, CYAN_GLOW)
    px(d, 32, 36, CYAN)

    bolt(d, 21, 47, SHADOW, BASE_LIGHT)
    bolt(d, 40, 47, SHADOW, BASE_LIGHT)

    # Twin autocannon barrels
    for bx in (21, 35):
        bevel_rect(d, (bx, 23, bx + 7, 30), BASE_LIGHT, STEEL, SHADOW)
        rect(d, (bx + 1, 25, bx + 6, 29), BASE_MID)
        hline(d, bx + 1, bx + 6, 25, EDGE_LIGHT)

        rect(d, (bx + 1, 11, bx + 6, 23), BASE_MID)
        vline(d, bx + 1, 11, 23, STEEL)
        vline(d, bx + 2, 11, 23, BASE_LIGHT)
        vline(d, bx + 3, 11, 23, BASE_MID)
        vline(d, bx + 4, 11, 23, BASE)
        vline(d, bx + 5, 11, 23, SEAM)
        vline(d, bx + 6, 11, 23, SHADOW)

        for fy in (15, 19):
            hline(d, bx + 1, bx + 6, fy, SHADOW)
            hline(d, bx + 1, bx + 6, fy - 1, EDGE_LIGHT)

        bevel_rect(d, (bx, 5, bx + 7, 10), BASE_LIGHT, STEEL, SHADOW)
        rect(d, (bx + 2, 5, bx + 5, 6), BASE_DARK)
        px(d, bx + 3, 5, SHADOW)
        px(d, bx + 4, 5, SHADOW)
        hline(d, bx + 1, bx + 6, 8, BASE_DARK)
        hline(d, bx + 2, bx + 5, 8, SHADOW)

    rect(d, (28, 17, 34, 19), BASE_DARK)
    hline(d, 28, 34, 17, BASE_LIGHT)
    hline(d, 28, 34, 19, SHADOW)
    return outline(image, SHADOW)


def draw_swarmer() -> Image.Image:
    """Generate the realistic quad missile launcher pod with 1px dark gunmetal outline."""
    image, d = clear(64, 64)
    chamfer_box(d, (8, 8, 55, 55), 5, BASE_MID)
    hline(d, 13, 50, 8, STEEL)
    vline(d, 8, 13, 50, STEEL)
    hline(d, 13, 50, 55, SHADOW)
    vline(d, 55, 13, 50, SHADOW)

    chamfer_box(d, (11, 11, 52, 52), 4, BASE)
    hline(d, 15, 48, 11, BASE_LIGHT)
    vline(d, 11, 15, 48, BASE_LIGHT)
    hline(d, 15, 48, 52, SHADOW)
    vline(d, 52, 15, 48, SHADOW)

    rect(d, (30, 12, 33, 51), BASE_DARK)
    rect(d, (12, 30, 51, 33), BASE_DARK)
    vline(d, 31, 12, 51, BASE_LIGHT)
    hline(d, 12, 51, 31, BASE_LIGHT)
    vline(d, 32, 12, 51, SHADOW)
    hline(d, 12, 51, 32, SHADOW)

    for cx, cy in ((13, 13), (35, 13), (13, 35), (35, 35)):
        chamfer_box(d, (cx, cy, cx + 15, cy + 15), 3, BASE_LIGHT)
        hline(d, cx + 3, cx + 12, cy, STEEL)
        vline(d, cx, cy + 3, cy + 12, STEEL)
        hline(d, cx + 3, cx + 12, cy + 15, SHADOW)
        vline(d, cx + 15, cy + 3, cy + 12, SHADOW)

        rect(d, (cx + 2, cy + 2, cx + 13, cy + 13), BASE_DARK)
        hline(d, cx + 2, cx + 13, cy + 2, SHADOW)
        vline(d, cx + 2, cy + 2, cy + 13, SHADOW)

        chamfer_box(d, (cx + 5, cy + 5, cx + 10, cy + 10), 2, BASE_MID)
        hline(d, cx + 6, cx + 9, cy + 5, EDGE_LIGHT)
        vline(d, cx + 5, cy + 6, cy + 9, EDGE_LIGHT)
        hline(d, cx + 6, cx + 9, cy + 10, SHADOW)
        vline(d, cx + 10, cy + 6, cy + 9, SHADOW)

        rect(d, (cx + 7, cy + 7, cx + 8, cy + 8), HOT_DARK)
        px(d, cx + 7, cy + 7, AMBER)

        px(d, cx + 2, cy + 7, SEAM)
        px(d, cx + 2, cy + 8, SEAM)
        px(d, cx + 13, cy + 7, SEAM)
        px(d, cx + 13, cy + 8, SEAM)

    for bx, by in ((9, 9), (52, 9), (9, 52), (52, 52)):
        bolt(d, bx, by, SHADOW, STEEL)
    bolt(d, 30, 30, SHADOW, STEEL)
    return outline(image, SHADOW)


def draw_lancer() -> Image.Image:
    """Generate the realistic twin-rail laser/beam turret with 1px dark gunmetal outline."""
    image, d = clear(64, 64)
    for bx in (16, 39):
        for fy in range(12, 36, 4):
            hline(d, bx - 2, bx + 10, fy, BASE_DARK)
            hline(d, bx - 2, bx + 10, fy - 1, EDGE_LIGHT)

        bevel_rect(d, (bx, 6, bx + 8, 38), BASE_LIGHT, STEEL, SHADOW)
        rect(d, (bx + 2, 8, bx + 6, 36), BASE_DARK)
        vline(d, bx + 2, 9, 35, GOLD)
        vline(d, bx + 6, 9, 35, BRONZE)
        vline(d, bx + 3, 10, 34, CYAN_DIM)
        vline(d, bx + 4, 10, 34, CYAN)
        vline(d, bx + 5, 10, 34, CYAN_DIM)
        vline(d, bx + 4, 10, 16, CYAN_GLOW)
        bevel_rect(d, (bx, 5, bx + 8, 8), BASE_LIGHT, STEEL, SHADOW)
        px(d, bx + 4, 5, CYAN_GLOW)

    chamfer_box(d, (12, 36, 51, 57), 6, BASE_LIGHT)
    hline(d, 18, 45, 36, STEEL)
    vline(d, 12, 42, 51, EDGE_LIGHT)
    hline(d, 18, 45, 57, SHADOW)
    vline(d, 51, 42, 51, SHADOW)

    rect(d, (18, 40, 45, 52), BASE_MID)
    rect(d, (24, 43, 39, 49), BASE_DARK)

    rect(d, (28, 44, 35, 48), CYAN_DIM)
    px(d, 31, 45, CYAN_GLOW)
    px(d, 32, 45, CYAN)

    rect(d, (13, 44, 16, 49), BRONZE)
    rect(d, (47, 44, 50, 49), BRONZE)

    edge_bolts(d, (16, 39, 47, 54), step=28, color=SHADOW, sheen=STEEL)
    return outline(image, SHADOW)


def draw_spectre() -> Image.Image:
    """Generate the realistic heavy twin autocannon turret with 1px dark gunmetal outline."""
    image, d = clear(128, 128)
    for bx in (27, 81):
        bevel_rect(d, (bx - 1, 66, bx + 20, 82), BASE_LIGHT, STEEL, SHADOW)
        rect(d, (bx + 1, 68, bx + 18, 80), BASE_MID)

        bevel_rect(d, (bx, 22, bx + 19, 66), BASE_MID, EDGE_LIGHT, SHADOW)
        vline(d, bx + 1, 22, 66, STEEL)
        vline(d, bx + 2, 22, 66, BASE_LIGHT)
        vline(d, bx + 17, 22, 66, SEAM)
        vline(d, bx + 18, 22, 66, SHADOW)
        for vy in (28, 38, 48, 58):
            for vx in (bx + 4, bx + 9, bx + 14):
                rect(d, (vx, vy, vx + 2, vy + 4), BASE_DARK)
                px(d, vx + 1, vy + 1, SHADOW)

        bevel_rect(d, (bx - 1, 8, bx + 20, 21), BASE_LIGHT, STEEL, SHADOW)
        for py in (11, 16):
            hline(d, bx + 1, bx + 18, py, BASE_DARK)
            hline(d, bx + 3, bx + 16, py, SHADOW)
        rect(d, (bx + 6, 8, bx + 13, 10), BASE_DARK)
        rect(d, (bx + 7, 8, bx + 12, 9), SHADOW)

    bevel_rect(d, (47, 46, 80, 50), BASE_LIGHT, STEEL, SHADOW)
    rect(d, (49, 48, 78, 49), BASE_DARK)

    chamfer_box(d, (16, 74, 111, 122), 10, BASE_LIGHT)
    hline(d, 26, 101, 74, STEEL)
    vline(d, 16, 84, 112, EDGE_LIGHT)
    hline(d, 26, 101, 122, SHADOW)
    vline(d, 111, 84, 112, SHADOW)

    bevel_rect(d, (22, 80, 48, 116), BASE_MID, EDGE_LIGHT, SHADOW)
    bevel_rect(d, (79, 80, 105, 116), BASE_MID, EDGE_LIGHT, SHADOW)

    for cy in (86, 94, 102):
        hline(d, 18, 25, cy, BRONZE)
        hline(d, 102, 109, cy, BRONZE)

    bevel_rect(d, (50, 82, 77, 112), BASE_DARK, EDGE_LIGHT, SHADOW)
    d.ellipse((56, 88, 71, 103), fill=BASE_MID)
    d.ellipse((58, 90, 69, 101), fill=CYAN_DIM)
    d.ellipse((61, 93, 66, 98), fill=CYAN)
    px(d, 63, 95, CYAN_GLOW)

    edge_bolts(d, (20, 78, 107, 118), step=28, color=SHADOW, sheen=STEEL)
    return outline(image, SHADOW)


def draw_meltdown() -> Image.Image:
    """Generate the realistic heavy thermal beam emitter turret with 1px dark gunmetal outline."""
    image, d = clear(128, 128)
    for bx in (24, 79):
        for fy in range(16, 76, 5):
            hline(d, bx - 3, bx + 27, fy, BASE_DARK)
            hline(d, bx - 3, bx + 27, fy - 1, EDGE_LIGHT)

        bevel_rect(d, (bx, 10, bx + 24, 78), BASE_LIGHT, STEEL, SHADOW)
        rect(d, (bx + 5, 14, bx + 19, 74), BASE_DARK)
        rect(d, (bx + 7, 16, bx + 17, 72), HOT_DARK)
        rect(d, (bx + 9, 18, bx + 15, 70), HOT)
        vline(d, bx + 12, 18, 40, AMBER)

        bevel_rect(d, (bx - 1, 8, bx + 25, 14), BASE_LIGHT, STEEL, SHADOW)
        rect(d, (bx + 7, 8, bx + 17, 10), BASE_DARK)
        px(d, bx + 12, 8, AMBER)

    bevel_rect(d, (49, 48, 78, 54), BASE_LIGHT, STEEL, SHADOW)
    rect(d, (51, 50, 76, 52), HOT_DARK)

    chamfer_box(d, (16, 74, 111, 122), 10, BASE_LIGHT)
    hline(d, 26, 101, 74, STEEL)
    vline(d, 16, 84, 112, EDGE_LIGHT)
    hline(d, 26, 101, 122, SHADOW)
    vline(d, 111, 84, 112, SHADOW)

    for ly in range(82, 114, 5):
        hline(d, 22, 44, ly, BASE_DARK)
        hline(d, 22, 44, ly - 1, BASE_LIGHT)
        hline(d, 83, 105, ly, BASE_DARK)
        hline(d, 83, 105, ly - 1, BASE_LIGHT)

    bevel_rect(d, (48, 80, 79, 112), BASE_DARK, EDGE_LIGHT, SHADOW)
    rect(d, (52, 84, 75, 108), HOT_DARK)
    rect(d, (56, 88, 71, 104), HOT)
    rect(d, (60, 92, 67, 100), AMBER)

    edge_bolts(d, (20, 78, 107, 118), step=28, color=SHADOW, sheen=STEEL)
    return outline(image, SHADOW)


def draw_foreshadow() -> Image.Image:
    """Generate the realistic 1x4 spinal railgun turret (64x256) with 1px dark gunmetal outline."""
    image, d = clear(64, 256)
    for bx, inner in ((14, 22), (41, 41)):
        bevel_rect(d, (bx, 8, bx + 8, 148), BASE_LIGHT, STEEL, SHADOW)
        vline(d, inner, 10, 146, BRONZE)
        vline(d, inner + (1 if bx == 14 else -1), 10, 146, GOLD)

    for y in range(14, 142, 12):
        bevel_rect(d, (23, y, 40, y + 6), BASE_DARK, EDGE_LIGHT, SHADOW)
        rect(d, (25, y + 1, 38, y + 5), CYAN_DIM)
        rect(d, (27, y + 2, 36, y + 4), CYAN)
        px(d, 31, y + 3, CYAN_GLOW)
        px(d, 32, y + 3, CYAN_GLOW)

    bevel_rect(d, (12, 6, 51, 10), BASE_LIGHT, STEEL, SHADOW)
    rect(d, (26, 6, 37, 7), BASE_DARK)

    for ry in (48, 88, 128):
        bevel_rect(d, (12, ry, 51, ry + 4), BASE_LIGHT, STEEL, SHADOW)
        bolt(d, 13, ry + 1, SHADOW, STEEL)
        bolt(d, 48, ry + 1, SHADOW, STEEL)

    chamfer_box(d, (8, 144, 55, 250), 8, BASE_LIGHT)
    hline(d, 16, 47, 144, STEEL)
    vline(d, 8, 152, 242, EDGE_LIGHT)
    hline(d, 16, 47, 250, SHADOW)
    vline(d, 55, 152, 242, SHADOW)

    bevel_rect(d, (12, 150, 51, 244), BASE_MID, EDGE_LIGHT, SHADOW)

    for fy in range(156, 188, 4):
        hline(d, 15, 48, fy, BASE_DARK)
        hline(d, 15, 48, fy - 1, BASE_LIGHT)

    bevel_rect(d, (16, 192, 47, 232), BASE_DARK, EDGE_LIGHT, SHADOW)
    rect(d, (20, 196, 43, 228), BRONZE)
    rect(d, (22, 198, 41, 226), GOLD)
    rect(d, (26, 202, 37, 222), AMBER)
    vline(d, 31, 202, 222, EDGE_LIGHT)

    bevel_rect(d, (14, 236, 49, 248), BASE_LIGHT, STEEL, SHADOW)
    d.ellipse((22, 238, 41, 246), fill=BASE_DARK)

    edge_bolts(d, (14, 148, 48, 246), step=22, color=SHADOW, sheen=STEEL)
    return outline(image, SHADOW)


def main() -> None:
    """Regenerate all block sprite PNGs (except hull.png) and verify output dimensions."""
    save("turret-base", draw_turret_base())
    save("scrap", draw_scrap())
    save("steel", draw_steel())
    save("nucleus", draw_nucleus())
    save("booster", draw_booster())
    # Note: hull.png is intentionally omitted to preserve the approved baseline reference.
    save("thruster", draw_thruster())
    save("battery", draw_battery())

    save("wedge-1x1", draw_right_triangle(64, 64))
    save("cap-1x1", draw_isosceles_triangle(64, 64))
    save("plate-2x2", draw_right_triangle(128, 128))
    save("fin-1x2", draw_isosceles_triangle(64, 128))
    save("spike-1x3", draw_isosceles_triangle(64, 192))

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
