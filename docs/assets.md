# Assets Spec

## Root

`core/assets` — relative to the `core` crate working directory (Bevy default).
The workspace-root `assets/` tree is not used and contains no shipped assets.

## Layout

```
assets/
  audio/bullet/      shoot-spectre.ogg
  config/base/       game.ron, block.ron
  config/mission/    quick-battle.ron
  config/starter/    interceptor.ron, raider.ron, b2.ron, blackbird.ron, aegis.ron, vanguard.ron
  config/            setting.ron
  shader/            shield.wgsl
  texture/block/     block + turret + core + wall sprites (PNG)
```

The procedural background in `scene/background.rs` loads no textures, so `texture/background/` does not exist.

Bevy paths used in block/item registration are relative to this root, e.g.
`texture/block/turret-base.png`, resolved as `AssetServer::load("texture/{…}")`.

## Conventions

- Filenames SHALL use lowercase kebab-case with a lowercase extension, e.g. `steel.png`, `shoot-spectre.ogg`.
- Mixed case, PascalCase, UUID stems, and uppercase extensions SHALL NOT be used.
- Bevy paths in `.ron` configs are case-sensitive and must match the on-disk filenames exactly.
- Sprites are square `GRID_SIZE`-aligned where feasible; block sprites render at
  `def.get_pixel_size()` (shape × `64 px`).
- Team-colored variants are drawn as full-size overlay sprites tinted via `team.color()` or HDR color multipliers for bloom.
- Sound effects are spatialized (`PlaybackSettings::REMOVE.with_spatial(true)`), scale
  `AUDIO_SCALE = 1/8000`.
- Block audio hookup is per-shot (`shoot-spectre.ogg`); no other blocks have sounds.

## Validation

A dev-only `AssetCheckPlugin` in `core/src/asset/` walks the asset tree at startup and warns for each filename that violates the convention. It is registered only under the `dev` feature in `main.rs` and never ships in production builds. Warnings do not fail startup.

## Used Assets

Referenced by `config/base/block.ron` and code:

- `texture/block/`: `battery`, `booster`, `cap-1x1`, `colaris`, `fin-1x2`, `foreshadow`, `hull`, `lancer`, `meltdown`, `nucleus`, `plate-2x2`, `scrap`, `spectre`, `spike-1x3`, `steel`, `swarmer`, `thruster`, `turret-base`, `wedge-1x1` (all `.png`); all block PNGs are generated deterministically by `tools/gen_block_art.py`
- `audio/bullet/shoot-spectre.ogg`
- `config/base/game.ron`, `config/base/block.ron`, `config/setting.ron`, `config/mission/quick-battle.ron`, `config/starter/{interceptor,raider,b2,blackbird}.ron`

`colaris.png` is shared by the `colaris` and `parallax` blocks; `docs/defects.md` #21 tracks this separately.

No font assets (UI uses the default font).
