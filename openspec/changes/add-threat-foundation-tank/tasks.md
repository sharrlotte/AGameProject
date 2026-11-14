## 1. Shared damage contract

- [x] 1.1 Add `DamageKind`, `Armor`, `Dying`, and `DeathResolutionSet` to `engine/src/common/mod.rs`
- [x] 1.2 Extend `DamageMessage` with `kind: DamageKind` and `hit_dir: Vec3`
- [x] 1.3 Add a serde-defaulted `damage_kind` to `WeaponDefinition` defaulting to `Ballistic`; carry it on `Projectile`
- [x] 1.4 Update projectile collision damage to pass the projectile kind and direction
- [x] 1.5 Update all `DamageMessage` constructors in tests and systems for the new fields
- [x] 1.6 Add unit tests for default ballistic kind and zero-direction handling

## 2. Archetype configuration

- [x] 2.1 Define `Behavior` (`Melee`, `Armor`), `SpawnPolicy`, and the extended `EnemyArchetypeData` fields
- [x] 2.2 Replace fixed `walker` / `runner` fields with `archetypes: Vec<EnemyArchetypeData>` plus global brain and steering tuning
- [x] 2.3 Implement `EnemiesConfig::validate` (unique kinds, required Walker, positive stats, finite behavior values, positive weights) and a kind lookup helper
- [x] 2.4 Rewrite `game/assets/config/enemies/enemies.ron` for Walker, Runner, and Tank
- [x] 2.5 Replace the hand-written duplicated `Default` with an empty roster and move enemy tests onto `assert_ron_config_parses!`
- [x] 2.6 Move per-kind scrap into `EnemyArchetypeData::scrap_value`, add `scrap` to `EnemyKilledMessage`, and remove `walker_scrap` / `runner_scrap` from `ExtractionConfig` and `extraction.ron`
- [x] 2.7 Add config parse and validation tests, including duplicate kind and invalid armor rejection

## 3. Enemy identity and spawning

- [x] 3.1 Add `EnemyKind::Tank` and a shared `label()` helper; update all exhaustive matches
- [x] 3.2 Refactor `spawn_enemy_entity` to accept an archetype and insert behavior components (`Melee`, `Armor`)
- [x] 3.3 Implement spawn eligibility by `unlock_secs`, weighted roll, per-archetype cap, and global cap
- [x] 3.4 Verify `spawn_scrap_on_enemy_killed` consumes `EnemyKilledMessage.scrap`
- [x] 3.5 Add tests for locked archetypes, weighted eligibility, per-kind caps, and global caps

## 4. Brain, steering, and melee

- [x] 4.1 Add `EnemyBrain` with `Idle`, `Alert`, `Engage`, `Stagger`, target, and alert timers
- [x] 4.2 Implement proximity aggro, last-known-position alert, and alert-to-idle transitions
- [x] 4.3 Implement damage-triggered engagement and the knockback stagger transition
- [x] 4.4 Implement steering for `Engage` and `Alert`, and zero steering for `Idle`, `Stagger`, and active wind-up
- [x] 4.5 Add the configurable separation term without a spatial grid
- [x] 4.6 Add the `Melee` capability with wind-up, reach check, cooldown, damage, and knockback
- [x] 4.7 Chain and scope the enemy systems correctly in `EnemiesPlugin` for `Playing` and `Arena`
- [x] 4.8 Add headless tests for brain transitions, separation, and melee wind-up

## 5. Death pipeline

- [x] 5.1 Make `apply_enemy_damage` apply armor mitigation and mark lethal targets `Dying` without despawning
- [x] 5.2 Emit `EnemyKilledMessage` once from the damage pass with kind and scrap
- [x] 5.3 Add death-effect and cleanup systems in `DeathResolutionSet`, ordered inside the enemies feature
- [x] 5.4 Exclude `Dying` enemies from targeting, steering, and melee
- [x] 5.5 Add tests for one kill message, deferred despawn, and ignored post-death damage

## 6. Tank archetype

- [x] 6.1 Author the Tank stats, armor behavior, melee behavior, spawn policy, and scrap value in `enemies.ron`
- [x] 6.2 Verify frontal ballistic mitigation, back damage, and bypass behavior with unit tests
- [x] 6.3 Verify the wind-up roots the Tank, misses when the target leaves reach, and respects cooldown
- [x] 6.4 Verify Tank mass keeps ordinary bullet knockback below the stagger threshold
- [x] 6.5 Verify the Tank spawns only after its `unlock_secs` and respects `max_active`

## 7. Player knockback

- [x] 7.1 Add a `Knockback` component to the survivor
- [x] 7.2 Accumulate clamped horizontal knockback in `apply_player_damage`
- [x] 7.3 Add knockback to movement and decay it with configurable values in `player.ron`
- [x] 7.4 Add tests for displacement, decay, additive movement, zero knockback, and ignored vertical impulse

## 8. Presentation

- [x] 8.1 Add cached primitive meshes and steel/trim materials for the Tank
- [x] 8.2 Split enemy visual assembly by kind so the Tank builds a torso, front plate, shoulders, head, and arms
- [x] 8.3 Convert `HitFlash` to store and restore every visual part
- [x] 8.4 Confirm facing rotates the whole assembly through the `VisualBody` root
- [x] 8.5 Verify a dev build renders Walker, Runner, and Tank without missing-material warnings

## 9. Hot reload

- [x] 9.1 Sync live enemy stats and behavior values from the reloaded roster
- [x] 9.2 Apply spawn policy and scrap changes to subsequent spawns and kills
- [x] 9.3 Log a warning instead of panicking when a behavior is added or removed structurally
- [x] 9.4 Add tests for value-level armor and spawn-policy reload behavior

## 10. Documentation and verification

- [x] 10.1 Update `docs/02-enemies-horde-ai-and-escalation.md` with the brain states, damage/armor contract, Tank, and spawn policy
- [x] 10.2 Update `docs/07-technical-architecture-save-and-roadmap.md` with the new enemy modules and system sets
- [x] 10.3 Run targeted checks only: enemy, combat, and player tests plus the affected RON parse tests
- [x] 10.4 Confirm every touched source file stays under 500 lines and follows the 1-word file convention
