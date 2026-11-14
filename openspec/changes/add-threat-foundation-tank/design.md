## Context

Bevy 0.20, a two-crate workspace (`engine`, `game`). Enemy simulation lives in `engine/src/enemies/` with
`spawn`, `ai`, and `damage` sub-features. `Enemy` and `FacingDirection` live in `ai/state.rs`;
`EnemyKind` is shared in `common` because messages and presentation consume it. Configuration is loaded
through `AppRonConfigExt::register_ron_config::<T>()` with async load and scoped hot reload.

Current constraints that shape this design:

- `apply_enemy_damage` subtracts health, emits `EnemyKilledMessage`, and despawns the entity in one system.
- `DamageMessage` has no damage kind or hit direction; `apply_player_damage` ignores `knockback`.
- `Projectile` is player-fired only; this change does not need enemy projectiles.
- Enemy AI is one direct-chase system with no state, no targeting memory, and no separation.
- `EnemiesConfig` hardcodes `walker` / `runner` fields and duplicates every authored value in
  `Default`. `ExtractionConfig` separately hardcodes per-kind scrap amounts.
- Presentation assembles each enemy from one cached capsule and `HitFlash` can only flash one mesh.
- Source files stay under 500 lines, sub-features use 1-word file names, and cross-crate ordering uses
  named `SystemSet`s from `common`.

## Goals / Non-Goals

**Goals:**

- Establish an archetype foundation that later archetypes can extend without rewriting damage, death,
  AI, or spawn code.
- Make directional damage, armor, wind-up attacks, and on-death hooks first-class.
- Ship the Tank as the proving-ground archetype.
- Keep all tuning in `game/assets/config/enemies/enemies.ron` with validation and hot reload.
- Remove duplicated hardcoded enemy values from Rust.

**Non-Goals:**

- No enemy-fired projectiles, acid pools, explosions, summons, or stealth.
- No vision cones, noise propagation, or line-of-sight checks.
- No spatial hash grid or flow-field pathfinding.
- No Blender enemy models; Tank uses primitive assemblies.
- No world generation, loot containers, objectives, or persistence.

## Decisions

### D1. Closed `EnemyKind` identity with a RON archetype roster

- **Choice:** Keep `EnemyKind` as the closed identity enum and add `Tank`. Replace fixed config fields
  with `archetypes: Vec<EnemyArchetypeData>` keyed by `kind`, each carrying stats, `spawn`, `scrap_value`,
  and a `behaviors: Vec<Behavior>` list.
- **Rationale:** The seven archetypes are a design contract with distinct silhouettes, messages, and
  presentation. Exhaustive matches catch every integration site when a kind is added. Behavior values
  and composition remain data-driven.
- **Alternatives considered:**
  - String/ID-keyed archetypes: more flexible for future elites, but every new behavior still needs code
    and messages/presentation lose exhaustive checking.
  - Keep fixed `walker` / `runner` fields: does not scale to seven and repeats per-kind code in the
    spawner, scraper, and hot-reload systems.
- **Validation:** unique kinds, walker present, positive stats and radii, finite behavior values,
  weights non-negative with at least one positive, `arc_deg` in `(0, 360]`, multipliers non-negative.

### D2. Brain state machine plus capability components

- **Choice:** Add `EnemyBrain { state, target, alert_timer, stagger_timer }` with
  `Idle | Alert | Engage | Stagger`, and compose runtime components from the archetype behavior list:
  `Melee`, `Armor`.
- **Rationale:** Decisions (brain) are separated from locomotion (steering) and from per-mechanic
  actions (melee). Spitter, Screamer, and Stalker add a capability module without touching the brain.
- **Alternatives considered:**
  - A single `match enemy.kind` system: compact for two kinds, unmaintainable at seven plus elites.
  - Trait objects for behaviors: fights the ECS query model and complicates hot reload.

### D3. Directional damage with damage kinds

- **Choice:** Extend `DamageMessage` with `kind: DamageKind` and `hit_dir: Vec3`. `DamageKind` lives in
  `common` next to `Health`; all guns default to `Ballistic` through `#[serde(default)]` on the weapon
  definition, so current `weapon.ron` entries remain valid.
- **Armor rule:** `front = facing.dot(-hit_dir) >= cos(arc_deg / 2)`; `mult = 1.0` when the kind is in
  `bypass`, otherwise `front_mult` or `back_mult`.
- **Rationale:** `hit_dir` remains valid after the source entity despawns, works for projectiles, melee,
  and future radial explosions, and keeps armor a pure function of message + target state.
- **Alternatives considered:**
  - Derive direction from `source` transform: fails for delayed explosions and despawned projectiles.
  - Per-kind armor code inside `apply_enemy_damage`: mixes Tank into a generic damage system.

### D4. Death pipeline with `Dying` and `DeathResolutionSet`

- **Choice:** On lethal damage, insert `Dying`, emit `EnemyKilledMessage` exactly once, and leave the
  entity alive. A new `DeathResolutionSet` in `common` runs after `DamageResolutionSet`: death effects
  resolve first, then `despawn_dead` removes `Dying` entities.
- **Rationale:** Exploder, drop tables, and future death VFX need the entity and its transform after
  death. It also gives one stable ordering contract across features.
- **Alternatives considered:**
  - Observer (`On<Add, Dying>`) for death effects: idiomatic in Bevy 0.20, but ordering with cross-feature
    effects is implicit; keep observers as an option for entity-local effects later.
  - Despawn immediately and carry everything in the message: impossible for effects that need the
    entity's components or children.
- **Ordering:** `DamageResolutionSet` contains both `apply_player_damage` and `apply_enemy_damage`.
  `DeathResolutionSet` runs after it inside `SimPostPhysicsSet`, before transform propagation. The
  existing reload cancellation already orders after `DamageResolutionSet` and remains valid.

### D5. Per-archetype spawn policy and scrap value

- **Choice:** Each archetype owns `spawn: { unlock_secs, weight, max_active }` and `scrap_value`.
  `spawn_enemies` filters by `RunStats.elapsed_secs >= unlock_secs`, removes kinds at their per-kind cap,
  then rolls once over the remaining weights. The global `max_active_start/end` cap stays.
- **Rationale:** Spawn pacing is a property of the threat, not a parallel table. `EnemyKilledMessage`
  carries `scrap`, so extraction no longer matches on kind and future elites scale automatically.
- **Alternatives considered:**
  - A separate tier table listing weights: duplicates archetype identity and drifts when kinds are added.
  - Keep per-kind scrap in `ExtractionConfig`: every new kind requires an extraction config change.

### D6. Remove duplicated `Default` values; parse RON in tests

- **Choice:** `EnemiesConfig::default()` becomes an empty roster with neutral pacing values. Tests that
  need authored archetypes use `assert_ron_config_parses!(EnemiesConfig)`.
- **Rationale:** The current `Default` is a second source of truth for every enemy stat. The weapon
  refactor already established the parse-in-tests pattern.
- **Risk control:** The loading gate populates `Res<EnemiesConfig>` before `Playing`; no gameplay system
  runs against the empty default.

### D7. Player knockback as a decaying component

- **Choice:** Add `Knockback { velocity }` to the survivor. `apply_player_damage` accumulates the impulse;
  `player_movement_and_aim` adds it to the velocity and decays it exponentially.
- **Rationale:** Movement overwrites `Velocity` every frame, so writing knockback directly would be
  erased. A separate decaying channel preserves player control while making heavy hits readable.
- **Alternatives considered:**
  - Input lock during knockback: removes control and needs a new state machine.
  - Add knockback to `Velocity` in movement: couples damage knowledge into the movement system.

### D8. Primitive Tank assembly and multi-part hit flash

- **Choice:** `presentation/scene/enemy/` assembles per-kind visuals from cached primitives. Tank is a
  `VisualBody` root with torso, front plate, shoulders, head, and arm children, all rotated by the root.
  `HitFlash` stores `parts: Vec<(Entity, Handle<StandardMaterial>)>`.
- **Rationale:** Primitive assemblies land behavior first without an asset pipeline dependency, and the
  multi-part flash is required for every future multi-mesh archetype.
- **Alternatives considered:**
  - A Tank GLB now: stronger silhouette, but blocks behavior work on art review and preview cycles.
  - One merged mesh: fewer parts, but requires authoring and does not exercise the multi-part path.

### D9. Separation steering now, spatial grid later

- **Choice:** Add a configurable neighbor separation term to steering using an O(n²) pass. No spatial
  grid in this change.
- **Rationale:** 80 enemies is 6,400 pair checks per tick — negligible. The grid becomes valuable with
  larger hordes, turrets, and AoE, which arrive after this foundation.
- **Guardrail:** Keep separation strength low and expose it in RON so feel can be tuned without code.

### D10. Hot reload is value-level, not structural

- **Choice:** Hot reload updates existing stat and behavior *values* on live enemies. Adding or removing a
  behavior component on live entities is not required; the change logs a warning and applies on the next
  deploy.
- **Rationale:** Avoids a command-heavy insert/remove pass and keeps the reload path predictable. Behavior
  composition changes are authoring-time changes, not combat-time tuning.

## Risks / Trade-offs

- [Risk] Aggro ignores walls, so enemies pursue through cover. → Accepted for the arena; line-of-sight
  perception lands with the Spitter change, before ranged attacks.
- [Risk] Directional armor depends on `FacingDirection`, which can be zero or stale. → Treat near-zero
  facing as frontal, normalize `hit_dir`, and cover front/back/bypass in tests.
- [Risk] Separation changes the feel of existing Walkers and Runners. → Low default strength, RON-tunable,
  plus a test that two overlapping enemies gain opposing lateral velocity.
- [Risk] Structural behavior-list hot reload is limited. → Document value-level reload; warn on
  composition changes.
- [Risk] Tank unlock timing is a test convenience (120s) versus the design's Tier 2 at 6:00. → Keep
  `unlock_secs` authored in RON and tune when the run length is real.
- [Risk] Adding `DamageMessage` fields touches every writer and test. → Compiler-guided migration; all
  current gun damage defaults to `Ballistic`.
- [Risk] `EnemyKind` exhaustiveness forces edits in spawn, AI, presentation, and tests. → Intentional;
  exhaustive matches are the safety net for future archetypes.
- [Risk] Death effects that deal damage resolve one frame later. → Acceptable for a blast fuse; document
  in the Exploder change rather than building a same-frame damage loop now.

## Migration Plan

1. Add `DamageKind`, `Armor`, `Dying`, and `DeathResolutionSet` to `common`; extend `DamageMessage` and
   propagate `kind` / `hit_dir` through the projectile and melee writers.
2. Replace `EnemiesConfig` with the validated archetype roster, `Behavior`, `SpawnPolicy`, and
   `scrap_value`; migrate `enemies.ron`; remove the duplicated `Default`.
3. Add the brain, capability components, melee wind-up, steering separation, tiered spawning, and the
   death pipeline; rewire `EnemiesPlugin` ordering.
4. Add `EnemyKind::Tank` and all compiler-guided matches; add Tank primitive visuals and multi-part hit
   flash.
5. Add player knockback and move scrap handling to `EnemyKilledMessage`.
6. Update `extraction.ron`, `player.ron`, docs, and specs; run the enemy/combat/player tests.

Rollback is a single change revert: the RON schema and `DamageMessage` fields are internal to this repo,
and no save data exists yet.

## Open Questions

- None blocking. Tank `unlock_secs`, separation strength, and knockback tuning are authored values to be
  adjusted after the first play test.
