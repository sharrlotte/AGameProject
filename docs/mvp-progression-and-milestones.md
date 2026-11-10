# Basic Playable & MVP Milestone Roadmap

## 1. Core Rule: Basic Playable First (Cut Complexity)
Before building procedural cities, room templates, 7 zombie types, drones, or complex base economies, we build a **Basic Playable Game** using **simple 3D primitives, minimal UI, and direct logic**.

### What We Explicitly Ignore For Now
- ❌ **No procedural world / room templates / roof cutaways** → Start with 1 static hand-placed 3D arena with simple box walls/cover.
- ❌ **No spatial hash grids or flow fields** → For <100 zombies in a small arena, direct chase + `bevy_rapier3d` collision is enough.
- ❌ **No complex weapon mods / attachment trees** → Start with 2 hardcoded/simple guns (Rifle + Shotgun).
- ❌ **No 7 zombie archetypes** → Start with 1–2 simple melee zombies (**Walker** + **Runner**).
- ❌ **No drones, traps, or turrets in Stage 1** → Add 1 simple Gun Turret only *after* the basic player-vs-zombie loop is fun.
- ❌ **No multi-room Home Base, Generator power budget, or save files** → Start with a simple in-memory "Between-Run Upgrade" screen + instant restart.
- ❌ **No vehicles or multiplayer** → Strictly single-player on foot.

---

## 2. Stage 1: Basic Playable Game (Build Order)
Every step below produces something runnable and testable right away.

### Milestone 1.0 — 3D Arena & Camera ✅
- Swapped `bevy_rapier2d` for `bevy_rapier3d` (`0.37.0`) with `TimestepMode::Interpolated` (`1/60s` physics + `TransformInterpolation`) and `PresentMode::AutoVsync`.
- Spawned a data-driven `50m × 50m` 3D ground plane, directional + global ambient lighting, perimeter walls, and 8 cover blocks from `game/assets/config/arena.ron` with zero-friction colliders (`world::ArenaPlugin`, `presentation::VisualsPlugin`).
- Set up a fixed-angle top-down 3D camera with smooth damped translation follow and subtle cursor aim-lead offset (`presentation::CameraPlugin`).
- **Done when:** The window opens cleanly at 60 FPS showing a lit 3D arena.

### Milestone 1.1 — Player Movement & Mouse Aiming ✅
- Spawned a 3D `Survivor` / `LocalPlayer` capsule with `bevy_rapier3d` dynamic `RigidBody` (locked Y translation and rotations, child `VisualBody` rotation) driven by `game/assets/config/player.ron` (`world::PlayerPlugin`).
- Added `WASD` 8-directional movement + `Shift` sprint with stamina + exhaustion delay, decoupled via `ActionIntent` (`presentation::InputPlugin`).
- Raycasted the mouse cursor onto the player's horizontal aim plane and rotated the survivor visual toward the cursor.
- **Done when:** Moving and aiming independently feels crisp and responsive.

### Milestone 1.2 — Simple Shooting & Reloading ✅
- Left-click fires physical 3D `Sensor` + `Ccd::enabled()` sphere projectiles in the aimed direction with slight random spread (multiplied while sprinting), loaded from `game/assets/config/weapons.ron` (`combat::CombatPlugin`).
- Added **2 weapons** (`1`: **Assault Rifle** full-auto `600 RPM`, `24` dmg, `30` mag, `1.8s` reload; `2`: **Shotgun** semi-auto `0.75s`, `8` pellets × `14` dmg, `8` mag, `2.2s` reload).
- Added magazine ammo tracking, `R` manual reload, auto-reload on empty trigger pull, and reload cancellation when swapping (`1`/`2`).
- **Done when:** You can shoot, swap between 2 distinct-feeling guns, and reload.

### Milestone 1.3 — Basic Zombies & Combat Feedback ✅
- Spawned **Walkers** (`4.2 m/s`, `75` HP, `15` dmg) and **Runners** (`7.8 m/s`, `35` HP, `10` dmg) along the arena edges from `game/assets/config/enemies.ron` (`enemies::EnemiesPlugin`).
- Zombies chase the nearest `Survivor` directly and deal contact damage on per-zombie attack cooldowns.
- Shots deal damage via `DamageMessage`, apply decaying knockback velocity, flash struck zombie meshes white (`0.09s`, `presentation::VisualsPlugin`), and emit `EnemyKilledMessage` at `0` HP.
- Added Player HP (`PlayerDiedMessage`) + **Game Over** overlay with **Upgrade Shop** and **Deploy Again** buttons (`presentation::MenusPlugin`).
- **Done when:** Fighting waves of Walkers and Runners is genuinely fun.

### Milestone 1.4 — Basic Extraction Loop & Minimal Upgrades ✅
- Dead zombies emit `EnemyKilledMessage` and drop **Scrap** pickups (`walker_scrap: 2`, `runner_scrap: 3`) with magnetic pull + walk-over collection via `ScrapCollectedMessage` (`extraction::ExtractionPlugin`, `game/assets/config/extraction.ron`).
- Added **Escalation Timer** in `enemies`: spawn interval (`1.5s -> 0.42s`) and active enemy cap (`35 -> 80`) scale smoothly over `240s` (`enemies.ron`).
- Added **1 Extraction Zone** (glowing floor pad at north edge): standing inside for `10.0s` (pausing when stepping out) emits `ExtractionCompletedMessage`, banks collected Scrap into `ScrapBank`, and opens the **Upgrade Shop** (dying loses un-extracted Scrap and opens Game Over with both **Upgrade Shop** and **Deploy Again**).
- Added **Upgrade Shop** (`extraction::UpgradesPlugin`, `game/assets/config/upgrades.ron`, `presentation::MenusPlugin`): spend banked Scrap on `+Weapon Damage`, `+Max HP`, and `+Move & Reload Speed`, then click **Deploy Again**.
- **Done when:** You have a complete playable game loop (Deploy → Fight → Loot → Extract/Die → Upgrade → Repeat).

### Milestone 1.5 — Minimal Playable HUD ✅
- Replaced temporary debug readout with a visual HUD (`presentation::HudPlugin`): **HP bar** + **Stamina bar**, **Weapon & Ammo / Reload progress bar**, **Scrap counter** (`run + banked`), **Run Timer + Kills**, and **Extraction Hold Progress bar**.
- **Done when:** Anyone can play the basic game and immediately understand what's happening.

---

## 3. Basic Playable Checklist (Summary)

| Feature | Stage 1 Implementation |
| :--- | :--- |
| **Physics / View** | `bevy_rapier3d`, top-down angled 3D camera, box/capsule primitives |
| **Movement** | `WASD` 8-dir + `Shift` sprint + mouse plane raycast aim |
| **Weapons** | 2 guns (Rifle + Shotgun), spread, mag + `R` reload, `1/2` swap |
| **Enemies** | Walker + Runner (direct chase, contact damage, knockback, hit flash) |
| **Loop** | Scrap drops, simple spawn timer escalation, 10s Extraction Pad |
| **Meta / UI** | Simple in-memory Upgrade menu (+Damage / +HP / +Reload) + minimal HUD |

---

## 4. Later Stages (Only After Stage 1 Is Fun)

```
  [Stage 1: Basic Playable]  ◀── WE ARE HERE (Core + Simple + Easy First)
              │
              ▼
  [Stage 2: Combat Depth]    ──▶ Crits, recoil/bloom, audio, Hanabi VFX, Tank + Spitter,
              │                  1 Gun Turret + Proximity Mine, JSON/RON weapon data
              ▼
  [Stage 3: World & Loot]    ──▶ Seeded street/building blocks, loot containers, backpack,
              │                  1 Primary Objective, Building Alarm, dusk/fog escalation
              ▼
  [Stage 4: Base & Saves]    ──▶ Armory/Workshop/Generator/Radio screens + JSON save files
              │
              ▼
  [Stage 5: Full Vision]     ──▶ Handcrafted room templates, roof cutaways, Exploder/Screamer/
                                 Stalker, Drones, all 7 biomes/objectives, Vehicles, Co-op
```
