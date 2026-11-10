# Game Overview & Core Loop

## 1. High-Level Vision
A **single-player-first (multiplayer-ready) top-down 3D zombie extraction shooter** built in **Bevy (Rust)**. The game blends the tense risk-reward looting and extraction of an extraction shooter with horde survival pressure, tactical deployable automation, and long-term home-base progression.

| Pillar | Target Experience |
| :--- | :--- |
| **Visual Style** | Stylized Top-Down 3D / 2.5D low-poly aesthetic (AI-assisted + free 3D assets, strong silhouettes, readable lighting, atmospheric fog/night). |
| **Physics & Perspective** | Full 3D rendering and true 3D collision/height via `bevy_rapier3d` (replacing the initial `bevy_rapier2d` setup) with an angled top-down camera. |
| **Session Pacing** | **15–25 minutes** per expedition — deep enough for exploration, multi-region traversal, and objectives, short enough for high replayability. |
| **Death Stakes** | **Roguelite standard**: Dying during an expedition loses all un-extracted run loot and temporary expedition progress, while **home base upgrades, blueprints, and stashed gear are kept**. |
| **Scope Strategy** | Single-player first, engineered with clean simulation/input separation so co-op multiplayer can be refactored in later without rewriting core gameplay. |

---

## 2. The 7-Step Core Loop

```
┌──────────────────────────────────────────────────────────────────────┐
│                         META-PROGRESSION HUB                         │
│   ┌──────────────┐                                ┌──────────────┐   │
│   │  1. PREPARE  │                                │  7. UPGRADE  │   │
│   │ Loadout,Ammo,│                                │ Craft, Lab,  │   │
│   │ Drones,Meds  │                                │ Base Power   │   │
│   └──────┬───────┘                                └───────▲──────┘   │
└──────────┼────────────────────────────────────────────────┼──────────┘
           │ Deploy                                         │ Return
           ▼                                                │
┌──────────────────────────────────────────────────────────────────────┐
│                     EXPEDITION (15–25 Minutes)                       │
│                                                                      │
│  ┌───────────┐    ┌───────────┐    ┌───────────┐    ┌────────────┐   │
│  │ 2. DEPLOY │───▶│3. EXPLORE │───▶│4. SCAVENGE│───▶│ 6. EXTRACT │   │
│  │ Insert    │    │ Navigate, │    │ Loot,     │    │ Call Evac, │   │
│  │ Zone Edge │    │ Clear Bldgs│   │ Objectives│    │ Hold Siege │   │
│  └───────────┘    └─────┬─────┘    └─────┬─────┘    └────────────┘   │
│                         │                │                  ▲        │
│                         ▼                ▼                  │        │
│                   ┌─────────────────────────────────────────┴────┐   │
│                   │             5. ESCALATION                    │   │
│                   │  More spawns • Elites • Hordes • Fog / Night │   │
│                   └──────────────────────────────────────────────┘   │
└──────────────────────────────────────────────────────────────────────┘
```

### Example Run Walkthrough (20-Minute Expedition)
1. **Prepare (Base, 2 min):** You equip an Assault Rifle (hollow-point mod) + Shotgun, pack 4 magazines each, 2 medkits, 1 Gun Turret, and a Scout Drone. Your Generator currently caps you at 2 active base modules, so you bring a basic loadout.
2. **Deploy (0:00):** You drop into the **Suburbs** edge of a seeded quarantine zone. Daylight visibility is clear.
3. **Explore & Scavenge (0:00–10:00):** You clear houses for food/scrap, send the Scout Drone ahead into the **Hospital**, and complete a *Supply Recovery* objective while fighting Walkers and Runners.
4. **Escalation Kicks In (10:00–16:00):** Dusk falls, fog thickens, and Spitters/Exploders start appearing. A migrating horde forces you to drop your Gun Turret in a choke point while you sprint into a pharmacy.
5. **Extract (16:00–20:00):** Overencumbered with rare Lab Tech, you reach an active extraction helipad during its **timed window**. Triggering extraction fires a flare that draws a final siege horde; you survive the 45-second holdout with traps and extract.
6. **Upgrade (Base):** Back at base, you research a Heavy Turret blueprint in the Laboratory and craft a suppressor in the Workshop.

---

## 3. Core Design Rules & Trade-offs

### 3.1 Why Full 3D (`bevy_rapier3d`) Changes the Game
Moving from `bevy_rapier2d` to `bevy_rapier3d` unlocks true verticality and spatial readability:
- **Height & Cover:** Low walls vs. full walls (shoot over waist-high concrete barriers, block line-of-sight behind tall buildings).
- **Multi-story / Elevation:** Highway overpasses, hospital ramps, underground lab stairs, and sloped terrain.
- **3D Projectiles & Arcs:** Lobbed Spitter acid, thrown grenades, turret elevation angles, and knockback trajectories.
- *Implementation note:* Mouse aiming raycasts from the camera onto a horizontal aim plane at the player's weapon height (or snaps onto valid 3D enemy hitboxes under the cursor).

### 3.2 Multiplayer-Ready Architecture (Without Building Netcode Yet)
To keep single-player fast to build while staying refactorable for co-op later:
- **No singleton `Player` assumptions in gameplay systems:** Always query `Query<..., With<PlayerControlled>>` or pass `Entity` targets rather than assuming `Single<&Transform, With<Player>>`.
- **Command-driven input:** Translate raw mouse/keyboard into an intent component (`PlayerInputIntent { move_dir, aim_point, fire, reload, sprint, deploy }`). In co-op later, remote clients simply replicate intents.
- **ID-tagged ownership:** Every bullet, turret, drone, and loot credit stores `Owner(Entity)` so kills, XP, and aggro work cleanly for multiple players.
- **Deterministic simulation boundary:** Keep physics, AI, and combat in `FixedUpdate`; keep camera, audio, particles, and UI purely reactive in `Update`.
