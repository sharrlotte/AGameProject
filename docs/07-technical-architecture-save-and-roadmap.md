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

### Current Large-Scale Multi-Crate Architecture (`Milestones 1.0–2.0`)
```
game/
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
│   ├── presentation/          # Client concern modules: camera/, input/, anim/, vfx/, scene/, ui/
│   └── game/                  # App assembly library crate: build_app() wiring all plugins, diagnostics & dev tools
├── tests/                     # Workspace-level headless integration tests
└── core/
    ├── assets/
    │   ├── config/            # Domain-grouped .ron configs (player/, combat/, enemies/, world/, extraction/, base/)
    │   └── models/            # 3D glTF/GLB assets (player.glb)
    └── src/main.rs            # Thin App entry point calling game::build_app().run() (default-members = ["core"])
```

- **Strict `Update` / `PostUpdate` Schedule Ordering:** `ConfigPlugin` hot-reload & `InputPlugin` (`before(SimUpdateSet)`) → `SimUpdateSet` (`Update`) → `ScenePlugin` attach/animate, `PlayerAnimPlugin` bind/director & `HudPlugin` (`after(SimUpdateSet)`) → `Rapier` `PhysicsSet` (`PostUpdate`) → `SimPostPhysicsSet` (`after(PhysicsSet::Writeback)`) → `DamageResolutionSet` → `AnimationSystems` → `PlayerIkSet` → player facing & `ScenePlugin` enemy facing / `VfxPlugin` hit-flash → `TransformSystems::Propagate`.

### Player Animation Module (`presentation/src/anim/`)
- **Layers:** lower-body locomotion blend, upper-body action blend (aim/reload), a reserved additive blend, and a full-body death override. Mask groups are explicit bone lists in `animation.ron` because `Hips` is the spine ancestor.
- **Blend:** eight directions from four cardinal clips by projecting local movement onto forward/back/left/right, normalized so Bevy's blend node receives a stable weight set. Walk/strafe playback scales with horizontal speed.
- **Director:** `DirectorState` per animation player; exponential weight fades from `locomotion_blend_secs` / `action_blend_secs`; reload clip seeks from `WeaponInventory::reload_progress()` so reload-speed upgrades stay synchronized.
- **Facing:** `anim/facing.rs` owns the player rule: aim by default, sprint velocity overrides while above the movement threshold, previous facing is kept when aim is invalid, and `facing_blend_secs` smooths the return after sprint.
- **IK:** analytic two-bone solver (`anim/ik.rs`) drives the left support hand to the rifle `Foregrip` only in combat stance and while alive; it composes local transforms from the visual root so it runs after `AnimationSystems` without stale `GlobalTransform`s.
- **Config:** clip roles, layer membership, masks, blend times, speed clamps, IK chain, and target socket live in `game/core/assets/config/animation.ron`. `visuals.ron` keeps only the model path, safety scale/offset, and move threshold. `stance_hold_secs` lives in `weapons.ron` because the simulation owns `CombatStance`.
- **Hot reload:** editing `animation.ron` rebuilds the graph from live `AnimationTargetId`s, re-registers masks, resets director state, and restarts looped clips. Editing `visuals.ron` reloads the glTF scene.
- **States:** director, IK, and facing run only in `Playing`/`GameOver`; asset preparation runs outside `Loading`.

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
