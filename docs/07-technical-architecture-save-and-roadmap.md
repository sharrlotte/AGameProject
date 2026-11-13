# Technical Architecture, Save System & Roadmap

## 1. Tech Stack & Crate Plan (Bevy 0.20)
Currently `game/Cargo.toml` configures workspace crates using `bevy 0.20.0` (`file_watcher`, `dynamic_linking`, `serialize`), `bevy_rapier3d 0.37.0`, `bevy_hanabi 0.20`, `bevy-inspector-egui 0.38`, `ron 0.12`, `serde`, `serde_json`, `rand 0.10`, `thiserror 2`, `uuid`, and `directories`.

| System | Crate / Solution | Notes |
| :--- | :--- | :--- |
| **3D Physics** | `bevy_rapier3d` *(replace `bevy_rapier2d`)* | True 3D colliders, raycasts, elevation, cover walls, projectile collision groups. |
| **Particles / VFX** | `bevy_hanabi` *(already installed)* | Muzzle flashes, acid pools, blood bursts, sparks, extraction flares. |
| **Input Mapping** | `leafwing-input-manager` *(recommended)* | Clean action-based inputs (`Move`, `Aim`, `Fire`, `Sprint`, `Deploy`), controller-ready & intent-friendly. |
| **Data Definitions** | `ron` / `serde_json` | Data-driven weapons, attachments, zombie archetypes, loot tables, room templates. |
| **World / Chunks** | Custom 3D chunk manager + spatial hash grid | Streams/culls static 3D meshes & colliders; powers $O(1)$ horde/turret queries. |
| **Audio** | Bevy Spatial Audio (or `bevy_kira_audio`) | 3D positional gunshots, Screamer shrieks, Exploder hisses, ambient wind/alarms. |

---

## 2. Multiplayer-Ready ECS Architecture (Single-Player First)

Even though Phase 0–2 are strictly single-player, organizing plugins along a **Simulation vs. Presentation** split makes adding co-op networking dramatically easier later:

```
game/core/src/
├── main.rs                  # App bootstrap, states (MainMenu, HomeBase, Loading, Expedition, Extracted, Dead)
├── sim/                     # Authoritative Simulation (FixedUpdate, multiplayer-portable)
│   ├── input_intent.rs      # PlayerInputIntent component (decouples raw input from character logic)
│   ├── player.rs            # Movement, stamina, health, active abilities
│   ├── combat.rs            # Weapons, recoil/spread, reload FSM, projectiles, damage, knockback
│   ├── enemies/             # 7 zombie archetypes, FSM, spatial grid, horde steering, escalation
│   ├── automation/          # Turrets, mines, electric traps, scout & combat drones
│   ├── world/               # Seeded generator, biome graph, room template assembler, chunks
│   ├── expedition/          # Objectives, random events, timed extraction windows
│   └── data/                # Serde structs for weapons, items, loot tables, recipes
├── base/                    # Home Base facilities, generator power budget, crafting, research
├── persistence/             # Save/load manager (base progression + exit-only expedition snapshot)
├── presentation/            # Pure client-side (Update): 3D camera, roof cutaways, VFX, audio, HUD/UI
└── vehicles/                # [Post-Core] Drivable vehicle physics & mounted interactions
```

### Current Architecture (Two-Crate Layout, post collapse-to-two-crates)

Workspace now has two crates: `engine` (all simulation + presentation + config + message + common) and `game` (binary, assets, tests). The previous multi-crate layout (`crates/*`) has been removed. All domain configs live in `engine::config`, cross-feature messages in `engine::message::messages`, run state in `engine::message::resources`.
```
├── Cargo.toml                 # Root workspace (default-members = ["game"])
├── crates/
│   ├── common/                # GameState, SimUpdateSet, SimPostPhysicsSet, DamageResolutionSet, tags, shared components
│   ├── config/                # RonConfig + AppRonConfigExt::register_ron_config, ConfigPlugin, loader, parse test macro
│   ├── message/               # Cross-feature typed Messages, shared progression & run resources (RunStats, ScrapBank, UpgradeLevels)
│   ├── data/                  # Compatibility facade re-exporting common, config, and message (CoreDataPlugin)
│   ├── sim/                   # Headless simulation feature crates (sub-features use 1-word files: mod.rs, config.rs, state.rs, system.rs)
│   │   ├── world/             # ArenaPlugin (arena/*), PlayerPlugin (player/*), ActionIntent (intent.rs)
│   │   ├── combat/            # CombatPlugin: weapon/, reload/, projectile/, stance/
│   │   ├── enemies/           # EnemiesPlugin: spawn/, ai/, damage/
│   │   ├── extraction/        # ExtractionPlugin (pickup/, zone/), RunFlowPlugin (flow/), UpgradesPlugin (upgrade/)
│   │   ├── automation/        # [Phase 2] Turrets, mines, electric traps, scout & combat drones
│   │   └── base/              # [Phase 2] Base facilities, generator power budget, crafting, research
│   ├── persistence/           # [Phase 2] Save/load manager (profile_base + active_expedition)
│   └── presentation/          # Client concern modules: camera/, input/, anim/, vfx/, scene/, ui/
└── game/                      # Runnable game crate (build_app() in src/lib.rs, thin src/main.rs, tests/)
    ├── assets/
    │   ├── config/            # Domain-grouped .ron configs (player/, combat/, enemies/, world/, extraction/, base/)
    │   └── models/            # 3D glTF/GLB assets (player.glb)
    ├── src/
    └── tests/
```

- **Strict `Update` / `PostUpdate` Schedule Ordering:** `ConfigPlugin` hot-reload & `InputPlugin` (`before(SimUpdateSet)`) → `SimUpdateSet` (`Update`) → `CameraPlugin` (`apply_camera_zoom`, `sync_camera_on_hot_reload`), `ScenePlugin` attach/animate, and `PlayerAnimPlugin` asset bind/director (`after(SimUpdateSet)`) → `Rapier` `PhysicsSet` (`PostUpdate`) → `SimPostPhysicsSet` (`after(PhysicsSet::Writeback)`) → `DamageResolutionSet` → `PlayerAnimSet::Drive` (before `AnimationSystems`) → weapon messages/pose (after `AnimationSystems`) → `PlayerAnimSet::Ik` → `PlayerAnimSet::Facing` → `CameraPlugin::follow_local_player`, `ScenePlugin` enemy facing & `VfxPlugin` hit-flash → `TransformSystems::Propagate`.

### Camera & Input (`presentation/src/camera/`, `presentation/src/input/`)
- **Config (`game/assets/config/camera.ron`):** `CameraConfig` owns `offset`, `follow_sharpness`, `aim_lead_factor`, `max_aim_lead`, and zoom parameters (`zoom_default`, `zoom_min`, `zoom_max`, `zoom_step`, `zoom_sharpness`, `zoom_pixel_divisor`) with async loading and hot-reload.
- **Zoom Flow:** `InputPlugin::gather_local_input` reads `MouseWheel` while `ControlLeft`/`ControlRight` is held and emits `CameraZoomMessage`; `CameraPlugin::apply_camera_zoom` updates `CameraZoom.target` on `MainCamera`; `follow_local_player` exponentially damps `CameraZoom.current` and scales `CameraConfig::offset` along the look ray so pitch remains fixed. Both modules use `Single` for singleton `MainCamera` and `PrimaryWindow` queries.

### Player Animation Module (`presentation/src/anim/`)
- **Layers:** `root` is an `Add` combiner over `base_blend` (full-body locomotion blend plus the `death` override) and `additive` (`Add` with a `LOWER_BIT` mask) for upper-body actions. Aim and reload are procedural, so the graph has no `aim`/`reload` clip nodes and no duplicated lower/upper locomotion blends. All clip nodes use `1.0` graph node weight (never `0.0`) so Bevy's multi-clip blend never divides `0.0 / 0.0` into `NaN`. Mask groups are explicit bone lists in `animation.ron` because `Hips` is the spine ancestor.
- **Locomotion:** `animation.ron` lists anchors as `(role, angle_deg)` in aim space (`walk_fwd` 0°, `strafe_r` 90°, `walk_back` 180°, `strafe_l` -90°). Each frame brackets the aim-space movement angle with the two nearest anchors and blends them, so diagonals work with four cardinal clips and authored diagonals can be added without Rust changes. A fixed-size `AnchorWeights` buffer avoids per-frame allocation, and missing clips are dropped once with a warning at bind time.
- **Phase & speed:** all anchors share one normalized phase clock; the dominant anchor seeks to that phase when it changes, preserving foot cadence through direction changes. Playback scales with horizontal speed (`walk_speed_ref`, `speed_scale_min/max`). Sprint uses an optional per-anchor `sprint_role` clip with a smoothed gait crossfade (`gait_blend_secs`) and blended phase rate (`sprint_speed_ref`); anchors without a sprint variant fall back to the sprint playback multiplier.
- **Director:** `DirectorState` per animation player holds idle/anchor weights, phase, dominant anchor, and death; motion gating uses `LocomotionConfig::move_speed_threshold`; death and `GameOver` stop and override the stack.
- **Facing:** `anim/facing.rs` is yaw-only always-aim. The body faces the cursor even while sprinting or backpedaling; shortest-arc exponential smoothing keeps the turn frame-rate independent, invalid aim keeps the previous yaw, and death suppresses changes.
- **Weapon presentation:** `anim/weapon/` spawns a chest-height anchor under `PlayerVisualRoot` and mounts the active profile from `game/assets/config/player/weapons.ron`, resolving `Grip`/`Foregrip`/`Magazine`/`Muzzle` once and caching the magazine rest transform. The weapon transform blends relaxed/aim offsets and adds sway, recoil (`WeaponFiredMessage`), reload tilt/drop (`WeaponInventory::reload_progress()`), and swap lower/raise (`WeaponSwappedMessage`).
- **IK:** analytic two-bone solver (`anim/ik.rs`) drives both arms; the right targets `Grip`, the left `Foregrip` (or `Magazine` during reload) with reach clamping, pole hints, hand alignment, and optional weighted/clamped shoulder settle. It composes local transforms from the visual root so it runs after `AnimationSystems` without stale `GlobalTransform`s.
- **Config:** `animation.ron` holds facing, aim, anchors, speed, masks, clip roles, and dual-chain IK; `weapons.ron` holds per-weapon models, sockets, poses, recoil, reload, and swap; `visuals.ron` keeps only the player model path and safety scale/offset.
- **Hot reload:** editing `animation.ron` rebuilds the graph from live `AnimationTargetId`s, re-registers masks, rebinds arms, resets director state/phase, and restarts looped clips. Editing `visuals.ron` reloads the glTF scene; editing `weapons.ron` remounts the active weapon.
- **States, Scenes & Dev Test Stage:** `GameState` (`Loading`, `Playing`, `GameOver`, `Upgrades`) controls the core loop while `Scene` (`Arena`, and `#[cfg(feature = "dev")] AnimTest`) scopes stage plugins/systems via `run_if(in_state(Scene::...))` and entity cleanup via Bevy's built-in `DespawnOnExit`. Director, IK, and facing run in `Playing` and `GameOver`; asset preparation runs outside `Loading`. `ArenaPlugin` owns `Scene::Arena`, while in `dev` builds `AnimTestPlugin` (`world::testanim`) owns `Scene::AnimTest` — an enemy-free, obstacle-free flat ground stage with player, weapons, camera, and HUD, toggled via `F8` or the HUD/menu `Anim Test Stage (F8)` button. Production builds (`--no-default-features`) strip `AnimTestPlugin`, `Scene::AnimTest`, and dev UI completely.

### Key Rules to Avoid Refactor Pain Later
1. **No `Single<..., With<Player>>` inside simulation systems:** Query `With<Survivor>` and link UI/camera via a local `LocalPlayer` marker.
2. **Intent-Driven Actions:** Input systems only write `ActionIntent`; gameplay systems consume `ActionIntent`.
3. **Explicit Ownership & Events:** Projectiles, deployables, and kills carry `Owner(Entity)` and emit typed Bevy events (`DamageEvent`, `NoiseEmittedEvent`, `LootExtractedEvent`).
4. **Seeded World Generation:** Store `expedition_seed: u64` + delta state (looted containers, dead bosses, built deployables) rather than serializing millions of static world transforms.

---

## 3. Save System Design (Auto-Save on Exit Only)
Per the **Roguelite + No Save-Scumming** requirement:

| Save File | Contents | When Written / Deleted |
| :--- | :--- | :--- |
| **`profile_base.json`** | Base facility levels, Generator power allocation, unlocked blueprints/research, Armory stash, Radio Room sectors, player stats. | Auto-saved on every base upgrade, loadout change, and immediately upon **Extraction** or **Death**. |
| **`active_expedition.json`** | `expedition_seed`, elapsed escalation timer, player position/HP/inventory/ammo, placed deployables, completed/active objectives, opened container IDs, active extraction schedules. | **Written ONLY on clean exit/quit mid-expedition.** Immediately **deleted** on resume (anti-save-scum), death, or extraction. |

*Why this works:* If life interrupts a 20-minute run, the player can quit and resume right where they left off—but they cannot reload an old save after dying or taking a bad fight.

---

## 4. Drivable Vehicles (Post-Core Addition)
Vehicles are intentionally deferred until on-foot combat, world gen, and base progression feel great:
- **Future Scope:** Scavenged/repaired trucks or buggies on **Highway** and **Industrial** roads, requiring fuel/parts, offering trunk storage for heavy loot + mobile headlights at night, balanced by loud engine noise attracting hordes.
- **Architectural Prep Now:** Keep roads/highways wide enough (e.g., 8–10m) in Stage 2 world generation and avoid hardcoding the player's movement collider size into world doorways/roads.

---

## 5. Phased Development Roadmap

| Phase | Milestone Goal | Deliverable Scope |
| :--- | :--- | :--- |
| **Phase 0: Core Feel Prototype** | *"Shooting & movement feel awesome in 3D"* | `bevy_rapier3d` switch, 3D top-down camera with aim-lead, 8-dir movement + sprint/stamina, 2–3 weapons (recoil, spread, reload, crits, knockback), Walkers + Runners, hit VFX/shake in a test arena. |
| **Phase 1: Minimum Extraction Loop** | *"15-min run works start-to-finish"* | Seeded room/street generator (2 biomes), loot containers, all 7 zombie archetypes (gradual rollout), Escalation timer + fog/night, Timed Extraction holdout siege, Death vs. Extract outcomes. |
| **Phase 2: Automation & Home Base** | *"Tactical gadgets + meta progression"* | Gun/Heavy Turrets, Mines, Electric Traps, Scout/Combat Drones, 7 Base facilities + Generator power budget, Workshop crafting, Armory loadouts, Auto-save on exit. |
| **Phase 3: Full Content & Variety** | *"Rich replayability"* | All 7 biomes, full handcrafted room template library + roof cutaways, all 7 mission objective types, 5 random events, full weapon/attachment roster, Polish & Audio. |
| **Phase 4: Post-Core Expansion** | *"Scale up"* | Drivable vehicles on Highways/Districts, Co-op multiplayer refactor using `ActionIntent` + `Owner(Entity)` foundations. |
