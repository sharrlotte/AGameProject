"""Generate chunky pixel-art PNGs for the triangle hull block definitions.

The alpha mask uses the same triangle half-plane inequalities as
`triangle_cells` in content/block/component.rs, so rendered art matches the
baked hitbox. Art is drawn at half resolution and upscaled 2x with NEAREST.
"""

from pathlib import Path

from PIL import Image

GRID_SIZE = 64
OUT_DIR = Path(__file__).resolve().parents[1] / "core" / "assets" / "texture" / "block"

DARK = (38, 42, 50)
LIGHT = (86, 94, 106)
ACCENT = (214, 158, 46)

SPECS = (
    ("wedge-1x1", 1, 1, "right"),
    ("cap-1x1", 1, 1, "isosceles"),
    ("plate-2x2", 2, 2, "right"),
    ("fin-1x2", 1, 2, "isosceles"),
    ("spike-1x3", 1, 3, "isosceles"),
)


def inside(base: str, px: float, py: float, width: float, height: float) -> bool:
    """Half-plane test in centered cell units; px,py are cell-relative offsets."""
    if base == "right":
        return px / width + py / height <= 0.0
    upper = px / width >= py / (2.0 * height) - 0.25
    lower = px / width <= 0.25 - py / (2.0 * height)
    return upper and lower


def shade(px: float, py: float, width: float, height: float, margin: float) -> tuple[int, int, int]:
    """Top-left highlight, bottom-right shadow, with an amber chamfer accent."""
    light = max(0.0, min(1.0, 0.5 + (-px / width + py / height) * 0.5))
    color = tuple(round(d + (l - d) * light) for d, l in zip(DARK, LIGHT))

    if margin < 0.16:
        color = tuple(round(c + (a - c) * 0.35) for c, a in zip(color, ACCENT))
    return color


def render(name: str, cells_x: int, cells_y: int, base: str) -> Image.Image:
    low_w, low_h = cells_x * GRID_SIZE // 2, cells_y * GRID_SIZE // 2
    size_x, size_y = float(low_w), float(low_h)
    pixels = []

    for row in range(low_h):
        for col in range(low_w):
            px = col + 0.5 - low_w / 2.0
            py = low_h / 2.0 - (row + 0.5)
            if not inside(base, px, py, size_x, size_y):
                pixels.append((0, 0, 0, 0))
                continue
            if base == "right":
                margin = -(px / size_x + py / size_y)
            else:
                margin = min(
                    px / size_x - py / (2.0 * size_y) + 0.25,
                    0.25 - py / (2.0 * size_y) - px / size_x,
                )
            pixels.append((*shade(px, py, size_x, size_y, margin), 255))

    low = Image.new("RGBA", (low_w, low_h))
    low.putdata(pixels)
    return low.resize((low_w * 2, low_h * 2), Image.Resampling.NEAREST)


def main() -> None:
    OUT_DIR.mkdir(parents=True, exist_ok=True)
    for name, cells_x, cells_y, base in SPECS:
        image = render(name, cells_x, cells_y, base)
        path = OUT_DIR / f"{name}.png"
        image.save(path)
        opaque = sum(1 for alpha in image.getchannel("A").get_flattened_data() if alpha == 255)
        print(f"{path.name}: {image.size[0]}x{image.size[1]}, opaque={opaque}")


if __name__ == "__main__":
    main()
