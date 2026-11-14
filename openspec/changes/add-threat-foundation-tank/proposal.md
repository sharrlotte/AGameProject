## Why

The combat loop currently has only Walkers and Runners, and both share one direct-chase behavior. The
enemy layer also cannot express the threats the design depends on: damage has no type or direction, so
armor cannot work; `apply_enemy_damage` despawns a zombie in the same system that kills it, so on-death
behavior cannot work; and the spawner has one scalar weight with no tier gating. Adding the remaining five
archetypes without fixing those foundations would force every archetype to fight the same limitations.

Tank is the smallest archetype that exercises the new foundation — armor, directional damage, a readable
wind-up attack, and heavy knockback — while remaining independent of world generation and enemy-fired
projectiles.

## What Changes

- Replace the fixed `walker` / `runner` fields in `EnemiesConfig` with a validated RON archetype roster.
  Each archetype carries stats, a behavior list, a spawn policy (`unlock_secs`, `weight`, `max_active`),
  and `scrap_value`. **BREAKING**: `game/assets/config/enemies/enemies.ron` schema changes.
- Add an `EnemyBrain` state machine (`Idle`, `Alert`, `Engage`, `Stagger`) with proximity and damage aggro,
  deaggro timing, and a stagger reaction. Locomotion becomes steering with separation.
- Extract combat capabilities into components driven by the archetype behavior list: `Melee` (damage,
  cooldown, wind-up, reach, knockback) and `Armor` (front/back multiplier, arc, bypass kinds).
- Extend `DamageMessage` with `kind: DamageKind` (`Ballistic`, `Explosive`, `Acid`, `Shock`) and
  `hit_dir: Vec3`. All existing guns default to `Ballistic`, so no weapon RON edits are required.
- Replace immediate enemy despawn with a death pipeline: damage inserts `Dying` and emits
  `EnemyKilledMessage` once; a new `DeathResolutionSet` resolves death effects and then despawns.
- Add the Tank archetype: high HP, slow movement, frontal ballistic mitigation, vulnerable back,
  wind-up melee slam, knockback resistance through mass, and 8 scrap.
- Add player knockback response so the Tank slam displaces the survivor instead of only reducing health.
- Replace the per-kind scrap fields in `ExtractionConfig` with `scrap_value` on the archetype, carried by
  `EnemyKilledMessage`. **BREAKING**: `game/assets/config/extraction/extraction.ron` loses
  `walker_scrap` / `runner_scrap`.
- Add primitive Tank visuals (torso, front plate, shoulders, head, arms) and make hit flash cover every
  part of a multi-mesh enemy.
- Remove the hand-written duplicated values in `EnemiesConfig::default()`; enemy tests read the authored
  RON through the existing parse helper.

Deferred to later changes: the neutral ordnance module and Spitter projectile, vision/noise senses,
Exploder/Screamer/Stalker, and the spatial hash grid.

## Capabilities

### New Capabilities

- `enemy-threats`: archetype roster and validation, brain state machine, damage kinds and directional
  armor, the death pipeline, tiered spawn policy, the Tank archetype, enemy primitive visuals, and hot
  reload of live enemy behavior.

### Modified Capabilities

- `player-combat-state`: taking damage with a knockback impulse now displaces the survivor through a
  decaying knockback response instead of being ignored.

## Impact

- Affected code: `engine/src/common/` (damage kinds, armor, `Dying`, `DeathResolutionSet`),
  `engine/src/config/{enemies,player,extraction}.rs`, `engine/src/enemies/{brain,ai,melee,damage,spawn}/`,
  `engine/src/combat/projectile/`, `engine/src/world/player/`,
  `engine/src/presentation/scene/{enemy,mesh,material}.rs`, `engine/src/presentation/vfx/flash.rs`.
- Affected assets: `game/assets/config/enemies/enemies.ron`,
  `game/assets/config/extraction/extraction.ron`, `game/assets/config/player/player.ron`.
- Affected docs: `docs/02-enemies-horde-ai-and-escalation.md`,
  `docs/07-technical-architecture-save-and-roadmap.md`.
- Unblocks: Spitter (needs directional damage and a death pipeline), Exploder (needs on-death effects),
  Screamer (needs brain states), Stalker (needs brain states), and later turret/AoE targeting.
