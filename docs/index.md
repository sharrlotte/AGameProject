# AGameProject Documentation

Spec-style documentation of the game as implemented in `core/src`. Documents
describe current behavior; the "Defects" sections record divergences between
current behavior and the apparent intended design. Implementation notes and
status journals do not belong here.

## Index

- [architecture.md](architecture.md) — workspace layout, plugin graph, schedule model, state model
- [build.md](build.md) — dev build profile, dynamic linking, check-first verification workflow
- [mission.md](mission.md) — battle-core scaffolding: Quick Battle, mission .ron, win/lose flow
- [simulation.md](simulation.md) — ships, blocks, grid, colliders, physics, drops
- [combat.md](combat.md) — turrets, targeting, shooting patterns, projectiles, damage
- [economy.md](economy.md) — items, storage, ammo, rarity
- [ai.md](ai.md) — AI ship controller, behaviors, ship movement
- [scenes.md](scenes.md) — Home, Game, Ship Editor, Pause, Settings, background
- [camera.md](camera.md) — projection, design frame, window mode, zoom, pan
- [save.md](save.md) — schematic format, world save format, persistence
- [ui.md](ui.md) — HUD, ship list, panels
- [controls.md](controls.md) — input bindings
- [assets.md](assets.md) — asset folder layout and naming rules
- [block-art.md](block-art.md) — block sprite art direction, lighting, continuity, palette and generator
- [defects.md](defects.md) — known defects in the current implementation (facts, not plans)
