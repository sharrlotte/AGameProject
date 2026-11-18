"""Generate every block sprite at final resolution with one unified art system.

All pieces share a single upper-left light source, bevel depth, 2-pixel minimum
detail width and palette so assembled hulls read as continuous armor. Triangle
masks use the same half-plane inequalities as the engine's triangle rasterizer,
keeping alpha aligned with baked hitboxes.
"""

from pathlib import Path

from PIL import Image, ImageDraw

OUT = Path(__file__).resolve().parents[1] / "core" / "assets" / "texture" / "block"

BASE_DARK = (38, 42, 50, 255)
BASE = (58, 64, 74, 255)
BASE_LIGHT = (72, 79, 90, 255)
EDGE_LIGHT = (86, 94, 106, 255)
STEEL = (104, 112, 124, 255)
SHADOW = (30, 34, 41, 255)
SEAM = (46, 51, 60, 255)

CYAN = (64, 190, 230, 255)
CYAN_DIM = (38, 112, 150, 255)
CYAN_GLOW = (132, 228, 255, 255)
BLUE = (48, 62, 90, 255)
BLUE_LIGHT = (66, 84, 118, 255)

GOLD = (186, 148, 72, 255)
GOLD_LIGHT = (214, 158, 46, 255)
BRONZE = (138, 106, 52, 255)
AMBER = (222, 166, 58, 255)

HOT = (206, 92, 58, 255)
HOT_DARK = (126, 52, 40, 255)

CLEAR = (0, 0, 0, 0)
DETAIL = 2


def new(width: int, height: int) -> tuple[Image.Image, ImageDraw.ImageDraw]:
    image = Image.new("RGBA", (width, height), CLEAR)
    return image, ImageDraw.Draw(image)


def opaque(width: int, height: int) -> tuple[Image.Image, ImageDraw.ImageDraw]:
    image = Image.new("RGBA", (width, height), BASE_DARK)
    return image, ImageDraw.Draw(image)


def save(name: str, image: Image.Image) -> None:
    OUT.mkdir(parents=True, exist_ok=True)
    image.save(OUT / f"{name}.png")


def line(
    draw: ImageDraw.ImageDraw,
    points: list[tuple[int, int]],
    fill: tuple[int, int, int, int],
    width: int = DETAIL,
) -> None:
    draw.line(points, fill=fill, width=max(DETAIL, width))


def dot(
    draw: ImageDraw.ImageDraw,
    x: int,
    y: int,
    color: tuple[int, int, int, int],
) -> None:
    draw.rectangle((x, y, x + DETAIL - 1, y + DETAIL - 1), fill=color)


def plate(
    draw: ImageDraw.ImageDraw,
    box: tuple[int, int, int, int],
    fill: tuple[int, int, int, int] = BASE,
    chamfer: int = 6,
) -> None:
    x0, y0, x1, y1 = box
    c = chamfer
    outline = [
        (x0 + c, y0),
        (x1 - c, y0),
        (x1, y0 + c),
        (x1, y1 - c),
        (x1 - c, y1),
        (x0 + c, y1),
        (x0, y1 - c),
        (x0, y0 + c),
    ]
    draw.polygon(outline, fill=fill)
    line(draw, [(x0 + c, y0), (x1 - c, y0)], EDGE_LIGHT)
    line(draw, [(x0, y0 + c), (x0, y1 - c)], EDGE_LIGHT)
    line(draw, [(x0 + c, y1), (x1 - c, y1)], SHADOW)
    line(draw, [(x1, y0 + c), (x1, y1 - c)], SHADOW)


def rivets(
    draw: ImageDraw.ImageDraw,
    box: tuple[int, int, int, int],
    inset: int = 8,
    color: tuple[int, int, int, int] = BRONZE,
) -> None:
    x0, y0, x1, y1 = box
    for x, y in (
        (x0 + inset, y0 + inset),
        (x1 - inset - DETAIL + 1, y0 + inset),
        (x0 + inset, y1 - inset - DETAIL + 1),
        (x1 - inset - DETAIL + 1, y1 - inset - DETAIL + 1),
    ):
        dot(draw, x, y, color)


def emissive(
    draw: ImageDraw.ImageDraw,
    box: tuple[int, int, int, int],
    color: tuple[int, int, int, int] = CYAN,
) -> None:
    x0, y0, x1, y1 = box
    draw.rectangle(box, fill=CYAN_DIM)
    draw.rectangle((x0 + DETAIL, y0 + DETAIL, x1 - DETAIL, y1 - DETAIL), fill=color)
    dot(draw, x0 + DETAIL, y0 + DETAIL, CYAN_GLOW)


def draw_block_2() -> Image.Image:
    image, d = opaque(64, 64)
    plate(d, (2, 2, 61, 61), chamfer=8)
    d.ellipse((14, 14, 49, 49), fill=BASE_DARK)
    d.ellipse((18, 18, 45, 45), fill=BASE_LIGHT)
    d.ellipse((24, 24, 39, 39), fill=BASE_DARK)
    rivets(d, (2, 2, 61, 61), 8, GOLD)
    return image


def draw_block_iron() -> Image.Image:
    image, d = opaque(64, 64)
    plate(d, (2, 2, 61, 61), BASE_DARK, chamfer=6)
    d.rectangle((10, 10, 53, 53), fill=BASE)
    line(d, [(10, 10), (53, 53)], EDGE_LIGHT, 4)
    line(d, [(10, 53), (53, 10)], SHADOW, 4)
    d.rectangle((26, 26, 37, 37), fill=BASE_LIGHT)
    rivets(d, (2, 2, 61, 61), 8, BRONZE)
    return image


def draw_block_steel() -> Image.Image:
    image, d = opaque(64, 64)
    plate(d, (2, 2, 61, 61), BASE, chamfer=6)
    line(d, [(8, 32), (55, 32)], SEAM)
    line(d, [(32, 8), (32, 26)], SEAM)
    line(d, [(32, 38), (32, 55)], SEAM)
    d.rectangle((24, 24, 39, 39), fill=BASE_LIGHT)
    line(d, [(24, 24), (39, 24)], EDGE_LIGHT)
    rivets(d, (2, 2, 61, 61), 8, BRONZE)
    return image


def draw_core() -> Image.Image:
    image, d = opaque(160, 160)
    plate(d, (4, 4, 155, 155), BASE, chamfer=24)
    plate(d, (24, 24, 135, 135), BASE_DARK, chamfer=20)
    line(d, [(80, 24), (80, 135)], GOLD, 4)
    line(d, [(24, 80), (135, 80)], GOLD, 4)
    d.ellipse((48, 48, 111, 111), fill=CYAN_DIM)
    d.ellipse((56, 56, 103, 103), fill=CYAN)
    d.ellipse((66, 66, 93, 93), fill=CYAN_GLOW)
    rivets(d, (4, 4, 155, 155), 18, BRONZE)
    return image


def draw_shielded_wall() -> Image.Image:
    image, d = opaque(64, 64)
    plate(d, (2, 2, 61, 61), chamfer=8)
    d.arc((12, 12, 51, 51), 210, 330, fill=CYAN_DIM, width=4)
    d.arc((18, 18, 45, 45), 210, 330, fill=CYAN, width=DETAIL)
    d.polygon([(32, 20), (42, 32), (32, 44), (22, 32)], fill=BASE_DARK)
    dot(d, 31, 31, CYAN_GLOW)
    rivets(d, (2, 2, 61, 61), 8, BRONZE)
    return image


def draw_surge_wall() -> Image.Image:
    image, d = opaque(64, 64)
    plate(d, (2, 2, 61, 61), chamfer=8)
    line(d, [(32, 8), (32, 55)], GOLD, 4)
    line(d, [(8, 32), (55, 32)], GOLD, 4)
    d.ellipse((22, 22, 41, 41), fill=BASE_DARK)
    d.ellipse((26, 26, 37, 37), fill=BASE_LIGHT)
    dot(d, 30, 30, EDGE_LIGHT)
    rivets(d, (2, 2, 61, 61), 8, BRONZE)
    return image


def draw_thruster() -> Image.Image:
    image, d = opaque(64, 64)
    plate(d, (2, 2, 61, 61), chamfer=8)
    d.polygon([(16, 16), (47, 16), (54, 50), (9, 50)], fill=BASE_DARK)
    d.polygon([(22, 24), (41, 24), (44, 44), (19, 44)], fill=CYAN_DIM)
    d.polygon([(26, 28), (37, 28), (38, 40), (25, 40)], fill=CYAN)
    dot(d, 31, 34, CYAN_GLOW)
    d.rectangle((10, 50, 53, 54), fill=BASE_LIGHT)
    rivets(d, (2, 2, 61, 61), 8, BRONZE)
    return image


def draw_titanium_wall() -> Image.Image:
    image, d = opaque(64, 64)
    plate(d, (2, 2, 61, 61), BLUE, chamfer=8)
    d.rectangle((14, 14, 49, 49), fill=BASE_DARK)
    d.rectangle((18, 18, 45, 45), fill=BLUE_LIGHT)
    emissive(d, (24, 20, 39, 42))
    line(d, [(14, 8), (49, 8)], EDGE_LIGHT)
    rivets(d, (2, 2, 61, 61), 8, GOLD)
    return image


def triangle_mask(width: int, height: int, base: str) -> Image.Image:
    mask = Image.new("L", (width, height), 0)
    pixels = []
    for row in range(height):
        for col in range(width):
            px = col + 0.5 - width / 2
            py = height / 2 - (row + 0.5)
            if base == "right":
                inside = px / width + py / height <= 0.0
            else:
                inside = (
                    px / width >= py / (2 * height) - 0.25
                    and px / width <= 0.25 - py / (2 * height)
                )
            pixels.append(255 if inside else 0)
    mask.putdata(pixels)
    return mask


def draw_triangle(width: int, height: int, base: str, detail: str) -> Image.Image:
    image, d = new(width, height)
    if base == "right":
        corners = [(0, height - 1), (width - 1, height - 1), (0, 0)]
    else:
        corners = [(0, height - 1), (width - 1, height - 1), (width // 2, 0)]
    d.polygon(corners, fill=BASE)
    line(d, [corners[0], corners[2]], GOLD)
    line(d, [corners[1], corners[2]], GOLD)
    line(d, [corners[0], corners[1]], EDGE_LIGHT)
    if detail == "panel":
        line(d, [(4, height - 6), (width - 6, height - 6)], SEAM)
        line(d, [(4, height - 12), (width - 12, height - 12)], SEAM)
    else:
        center_x = width // 2
        line(d, [(center_x, height - 6), (center_x, max(2, height // 2))], CYAN_DIM)
        dot(d, center_x - 1, max(2, height // 2), CYAN)
    image.putalpha(triangle_mask(width, height, base))
    return image


def draw_lancer() -> Image.Image:
    image, d = new(64, 64)
    d.rectangle((18, 6, 28, 46), fill=BASE_LIGHT)
    d.rectangle((36, 6, 46, 46), fill=BASE_LIGHT)
    d.rectangle((20, 10, 26, 42), fill=BASE_DARK)
    d.rectangle((38, 10, 44, 42), fill=BASE_DARK)
    line(d, [(23, 12), (23, 40)], CYAN)
    line(d, [(41, 12), (41, 40)], CYAN)
    plate(d, (14, 46, 50, 56), BASE, chamfer=4)
    dot(d, 31, 51, GOLD)
    return image


def draw_colaris() -> Image.Image:
    image, d = new(64, 64)
    d.rectangle((14, 16, 49, 50), fill=BASE)
    d.rectangle((22, 6, 30, 26), fill=BASE_LIGHT)
    d.rectangle((36, 6, 44, 26), fill=BASE_LIGHT)
    d.rectangle((24, 10, 28, 22), fill=BASE_DARK)
    d.rectangle((38, 10, 42, 22), fill=BASE_DARK)
    emissive(d, (28, 30, 36, 42))
    line(d, [(14, 16), (49, 16)], EDGE_LIGHT)
    rivets(d, (14, 16, 49, 50), 6, BRONZE)
    return image


def draw_swarmer() -> Image.Image:
    image, d = new(64, 64)
    d.rectangle((10, 10, 53, 53), fill=BASE)
    for x, y in ((18, 18), (38, 18), (18, 38), (38, 38)):
        d.rectangle((x, y, x + 8, y + 8), fill=BASE_DARK)
        dot(d, x + 3, y + 3, HOT)
        dot(d, x + 5, y + 5, HOT_DARK)
    line(d, [(10, 10), (53, 10)], EDGE_LIGHT)
    rivets(d, (10, 10, 53, 53), 6, BRONZE)
    return image


def draw_spectre() -> Image.Image:
    image, d = new(128, 128)
    d.rectangle((32, 10, 50, 88), fill=BASE_LIGHT)
    d.rectangle((78, 10, 96, 88), fill=BASE_LIGHT)
    d.rectangle((36, 14, 46, 84), fill=BASE_DARK)
    d.rectangle((82, 14, 92, 84), fill=BASE_DARK)
    line(d, [(40, 16), (40, 82)], GOLD)
    line(d, [(86, 16), (86, 82)], GOLD)
    plate(d, (24, 84, 104, 120), BASE, chamfer=8)
    d.rectangle((54, 94, 74, 110), fill=BASE_DARK)
    dot(d, 63, 101, CYAN)
    rivets(d, (24, 84, 104, 120), 8, BRONZE)
    return image


def draw_meltdown() -> Image.Image:
    image, d = new(128, 128)
    d.rectangle((32, 8, 50, 82), fill=BASE_LIGHT)
    d.rectangle((78, 8, 96, 82), fill=BASE_LIGHT)
    d.rectangle((36, 12, 46, 78), fill=HOT_DARK)
    d.rectangle((82, 12, 92, 78), fill=HOT_DARK)
    line(d, [(40, 14), (40, 76)], HOT)
    line(d, [(86, 14), (86, 76)], HOT)
    plate(d, (24, 82, 104, 120), BASE, chamfer=8)
    emissive(d, (56, 92, 72, 110), HOT)
    rivets(d, (24, 82, 104, 120), 8, BRONZE)
    return image


def draw_foreshadow() -> Image.Image:
    image, d = new(128, 128)
    d.rectangle((54, 6, 72, 104), fill=BASE_LIGHT)
    d.rectangle((58, 10, 68, 100), fill=BASE_DARK)
    line(d, [(62, 12), (62, 98)], CYAN_DIM, 4)
    line(d, [(62, 24), (62, 88)], CYAN)
    line(d, [(50, 16), (50, 100)], GOLD, 4)
    line(d, [(76, 16), (76, 100)], GOLD, 4)
    plate(d, (42, 100, 86, 124), BASE, chamfer=6)
    dot(d, 62, 111, CYAN_GLOW)
    return image


def main() -> None:
    save("turret-base", draw_block_2())
    save("scrap", draw_block_iron())
    save("steel", draw_block_steel())
    save("nucleus", draw_core())
    save("booster", draw_shielded_wall())
    save("hull", draw_surge_wall())
    save("thruster", draw_thruster())
    save("battery", draw_titanium_wall())

    save("wedge-1x1", draw_triangle(64, 64, "right", "panel"))
    save("cap-1x1", draw_triangle(64, 64, "isosceles", "core"))
    save("plate-2x2", draw_triangle(128, 128, "right", "panel"))
    save("fin-1x2", draw_triangle(64, 128, "isosceles", "panel"))
    save("spike-1x3", draw_triangle(64, 192, "isosceles", "core"))

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
