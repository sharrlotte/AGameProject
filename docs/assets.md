# Assets Spec

## Root

`game/core/assets` — relative to the `core` crate working directory (Bevy default).
`game/assets/` at workspace root is empty; do not populate it.

## Layout

```
assets/
  audio/bullet/      shoot-spectre.ogg
  texture/block/     block + turret + core + wall sprites (PNG)
  texture/background/ (unused; background is procedural)
```

Bevy paths used in block/item registration are relative to this root, e.g.
`texture/block/block-2.png`, resolved as `AssetServer::load("texture/{…}")`.

## Conventions

- **Case-sensitive** filenames; `block-steel.PNG` (mixed case) is not found when addressed
  as `block-steel.png` — current state.
- Sprites are square `GRID_SIZE`-aligned where feasible; block sprites render at
  `def.get_pixel_size()` (shape × `64 px`).
- Team-colored variants are drawn as full-size overlay sprites tinted via `team.color()`
  or HDR color multipliers for bloom.
- Sound effects are spatialized (`PlaybackSettings::REMOVE.with_spatial(true)`), scale
  `AUDIO_SCALE = 1/8000`.
- Block audio hookup is per-shot (`shoot-spectre.ogg`); no other blocks have sounds.

## Unused assets present

Background PNGs (`texture/background/background*.png`) and nebula sprites in
`texture/block/` (nebula, stars, titanium/ripple/shielded walls, block-scrap) are unused by
current registration data.

## Missing / soft references

- `texture/block/block-steel.png` (needs rename to match code).
- `texture/block/core-nucleus-team.png` referenced by Core registration — present.
- `audio/bullet/shoot-spectre.ogg` — present.
- No font assets (UI uses default font).
