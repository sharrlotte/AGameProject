# Assets Spec

## Root

`core/assets` — relative to the `core` crate working directory (Bevy default). There is
no `assets/` directory at the workspace root and `core/config/` does not exist; the asset
root is the only place assets are loaded from or written back to (settings write-back
uses `BEVY_ASSET_ROOT`, defaulting to `assets` relative to CWD).

## Layout

```
assets/
  audio/bullet/       shoot-spectre.ogg
  config/base/        game.ron, block.ron
  config/mission/     quick-battle.ron
  config/starter/     interceptor.ron, raider.ron
  config/setting.ron  volume + bloom (written back on change)
  texture/block/      block + turret + core + wall sprites (PNG)
```

Bevy paths used in block/item registration are relative to this root, e.g.
`texture/block/block-2.png`, resolved as `AssetServer::load("texture/{…}")`.

## Conventions

- **Case-sensitive** filenames; `block-steel.PNG` (mixed case) is not found when
  addressed as `block-steel.png` — open defect, see defects.md.
- Sprites are square `GRID_SIZE`-aligned where feasible; block sprites render at
  `def.get_pixel_size()` (shape × `64 px`).
- Team-colored variants are drawn as full-size overlay sprites tinted via `team.color()`
  or HDR color multipliers for bloom.
- Sound effects are spatialized (`PlaybackSettings::REMOVE.with_spatial(true)`), scale
  `AUDIO_SCALE = 1/8000`.
- Block audio hookup is per-shot (`shoot-spectre.ogg`); no other blocks have sounds.

## Unused assets present

Never referenced by code or any `.ron` — safe deletion candidates:

- `texture/background/027078b0-a55c-4139-bd9c-c17709b26df7.png`
- `texture/background/background1.png`
- `texture/background/background2.png`
- `texture/block/block-scrap.png`
- `texture/block/Nebula1.png`
- `texture/block/Nebula2.png`
- `texture/block/Nebula3.png`
- `texture/block/ripple.png`
- `texture/block/Stars.png`

Backgrounds are never loaded; the parallax starfield is procedural solid-color sprites
(`scene/background.rs`).
`titanium-wall-large.png` (`battery`) and `shielded-wall.png` (`booster`) ARE referenced
by `block.ron` and are in use.

## Missing / soft references

- `block-steel.PNG` → rename to `block-steel.png` to match `block.ron` (see defects.md).
- `texture/block/core-nucleus-team.png` referenced by Core registration — present.
- `audio/bullet/shoot-spectre.ogg` — present.
- No font assets (UI uses default font).
