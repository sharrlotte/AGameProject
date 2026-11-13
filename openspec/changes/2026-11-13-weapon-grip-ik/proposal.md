## Why

Holding a weapon looks wrong: the right and left hands use one shared orientation offset, palms face the
wrong way, wrists twist, fingers stay open, and the arms only approximate the grip because IK targets are
derived from a single socket rotation. Adding a new gun currently means adding weapon-specific code or
hand-tuned offsets in animation systems.

## What Changes

- Replace the single shared hand offset and per-hand pole vectors with a data-driven, per-weapon grip
  configuration authored in `weapons.ron`.
- Give each hand an optional `position`, `finger`, `palm`, `elbow`, `fingers`, `wrist_deg`, and
  `align_deg`, merged field by field over a grip derived from the weapon's `Grip`/`Foregrip` sockets.
- Resolve the weapon's forward/up/right basis from its socket frames so omitted grip fields are sensible
  for any new weapon without code changes.
- Replace the IK target type with weapon-space hand frames and per-hand wrist trims, and add elbow poles
  that come from the grip instead of the animation catalog.
- Move finger curl data into the grip config and keep the catalog curls as per-hand fallbacks.
- Support one-handed weapons with `support: None`, and fall back safely when a foregrip socket is too
  close to the grip.
- Add `model_space`/`model_rotation_deg` so converted GLBs authored as `-Y` forward, `+Z` up map into
  weapon space without ad-hoc rotations.
- Run idle relaxation before IK so the arm solve targets the relaxed body.
- Add a dev-only IK gizmo (`dev-ik-gizmo`) for calibration.

## Capabilities

### New Capabilities

- (none)

### Modified Capabilities

- `player-animation`: dual-hand IK now solves to resolved per-hand grip frames, applies per-hand finger
  curls, and relaxes the idle pose before IK.
- `weapon-presentation`: weapon profiles gain model-space mapping, socket overrides, and a data-driven
  grip configuration.

## Impact

- Affected code: `crates/presentation/src/anim/grip/` (new), `anim/ik/`, `anim/weapon/catalog/`,
  `anim/weapon/sockets.rs` (new), `anim/weapon/mount.rs`, `anim/weapon/pose.rs`, `anim/weapon/state.rs`,
  `anim/idle.rs`, `anim/mod.rs`.
- Affected config: `game/assets/config/player/weapons.ron`, `game/assets/config/player/animation.ron`.
- Presentation only; the simulation remains authoritative for weapon state, projectile origin, and
  damage.
- Adding a weapon requires only a new profile and grip entry, no Rust changes.
