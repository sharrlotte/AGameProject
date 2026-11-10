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

### Milestone 1.0 — 3D Arena & Camera
- Swap `bevy_rapier2d` for `bevy_rapier3d` in `game/core/Cargo.toml`.
- Spawn a flat 3D ground plane, basic directional light, and a few colored 3D box walls/cover blocks.
- Set up an angled top-down 3D camera that smoothly follows the player.
- **Done when:** The window opens cleanly at 60 FPS showing a lit 3D arena.

### Milestone 1.1 — Player Movement & Mouse Aiming
- Spawn a simple 3D player capsule/box with `bevy_rapier3d` collider.
- Add `WASD` 8-directional movement + simple `Shift` sprint (with basic stamina).
- Raycast the mouse cursor onto the player's horizontal plane and rotate the player toward the cursor.
- **Done when:** Moving and aiming independently feels crisp and responsive.

### Milestone 1.2 — Simple Shooting & Reloading
- Left-click fires simple 3D projectile spheres/capsules (or fast raycasts) in the aimed direction with slight random spread.
- Add **2 simple guns** (`1`: **Assault Rifle** full-auto, `2`: **Shotgun** multi-pellet burst).
- Add magazine ammo count + `R` reload timer.
- **Done when:** You can shoot, swap between 2 distinct-feeling guns, and reload.

### Milestone 1.3 — Basic Zombies & Combat Feedback
- Spawn **Walkers** (steady chase) and **Runners** (fast, low HP) from the edges of the arena.
- Zombies chase the player directly and deal contact damage on a short attack cooldown.
- Shots deal damage, apply simple knockback impulse, flash the enemy mesh white briefly, and despawn at 0 HP.
- Add Player HP + simple Game Over screen with a **Restart** button.
- **Done when:** Fighting waves of Walkers and Runners is genuinely fun.

### Milestone 1.4 — Basic Extraction Loop & Minimal Upgrades
- Dead zombies (or simple map crates) drop **Scrap** pickups you walk over to collect.
- Simple **Escalation Timer**: spawn rate slowly increases over a 3–5 minute test run.
- **1 Extraction Zone** (glowing floor pad): stand inside for 10 seconds to **Extract & Win**, keeping your collected Scrap (dying loses un-extracted Scrap).
- **Simple Upgrade Screen** between runs: spend Scrap on `+Weapon Damage`, `+Max HP`, or `+Move/Reload Speed`, then click **Deploy Again**.
- **Done when:** You have a complete playable game loop (Deploy → Fight → Loot → Extract/Die → Upgrade → Repeat).

### Milestone 1.5 — Minimal Playable HUD
- Display only essentials: **HP bar**, **Ammo / Reloading indicator**, **Scrap counter**, **Run Timer**, and **Extraction Hold Progress**.
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
