# Enemies, Horde AI & Escalation

## 1. Design Philosophy: Distinct Combat Problems
Zombies are **not** HP sponges with different numbers. Every archetype exists to force a specific player decision—reposition, prioritize target, switch weapons, or use a deployable.

---

## 2. Zombie Archetypes (Full 7-Type Roster)

| Archetype | Silhouette / Visual Cue | Combat Behavior | Tactical Problem & Counterplay |
| :--- | :--- | :--- | :--- |
| **1. Walker** | Hunched, shambling, muted colors | Slow pursuit; dangerous in large packs; surrounds player if ignored. | **Resource & Space Pressure:** Drains ammo and slowly closes exits. Counter with kiting, piercing rifles, or Gun Turrets. |
| **2. Runner** | Lean, erratic sprint posture | High speed, low HP, lunges at close range; zig-zags slightly. | **Reaction & Stagger Check:** Punishes slow reloads or tunnel vision. Counter with Shotguns (knockback) or Electric Traps. |
| **3. Tank** | Massive, armored/bulky torso | Slow-medium speed, heavy melee slam, high knockback resistance, frontal armor reduces bullet damage. | **Armor & Positioning Check:** Walks straight through basic Gun Turret fire. Counter by flanking its weak back, Heavy Turrets, or explosives. |
| **4. Spitter** | Glowing green/yellow sacs | Keeps medium distance; telegraphs and lobs 3D arcing acid projectiles that leave temporary hazard pools. | **Cover & Movement Check:** Forces you out of static camping spots and threatens stationary turrets. Counter with DMR/precision priority fire. |
| **5. Exploder** | Swollen, pulsing orange pustules | Moderate speed; glows/hisses loudly before detonating on proximity or death, dealing AoE damage + knockback (damages other zombies too!). | **Trigger Discipline & Spacing:** Punishes spraying wildly in tight hallways. Counter by shooting it from afar to use it as a living bomb against its own horde! |
| **6. Screamer** | Gaunt, elongated jaw, pale | Avoids direct melee; when spotting the player, emits a visible shockwave shriek that alerts & spawns nearby hordes. | **Priority Target:** Must be killed before it finishes its wind-up scream, or the room turns into a swarm. Suppressors help prevent early aggro. |
| **7. Stalker** | Low-profile, shadowy/translucent | Semi-invisible at range; circles toward player's back/flanks; decloaks with a screech just before pouncing. | **Awareness & Tech Check:** Causes ambush panic in dark buildings. Counter with Scout Drones, Pulse Scans, flashlight beam, or Proximity Mines at your back. |

Walker, Runner, Tank, Spitter, and Exploder are implemented. Screamer and Stalker are planned.

---

## 2.1 Implemented Threat Set

| Archetype | Stats | Behaviors | Spawn Policy |
| :--- | :--- | :--- | :--- |
| **Walker** | 75 HP, 4.2 m/s, mass 1.0, radius 0.45, half-height 0.45 | `Melee` 15 dmg, 0.9 s cooldown, no wind-up, 0.25 reach | Unlocked at 0 s, weight 0.70, `max_active` 60, scrap 2 |
| **Runner** | 35 HP, 7.8 m/s, mass 0.65, radius 0.35, half-height 0.40 | `Melee` 10 dmg, 0.7 s cooldown, no wind-up, 0.25 reach | Unlocked at 0 s, weight 0.30, `max_active` 40, scrap 3 |
| **Tank** | 420 HP, 3.1 m/s, mass 6.0, radius 0.75, half-height 0.70 | `Armor` 0.30 front / 1.0 back, 120° arc, `Explosive` bypass; `Melee` 32 dmg, 1.6 s cooldown, 0.45 s wind-up, 0.35 reach, 7.5 knockback | Unlocked at 120 s, weight 0.18, `max_active` 6, scrap 8 |
| **Spitter** | 90 HP, 3.6 m/s, mass 0.9, radius 0.42, half-height 0.50 | `Ranged` 8 dmg per pool tick, 2.6 s cooldown, 0.55 s wind-up, 14 m reach, 16 m/s cap, 35° arc; pool 2.5 m radius, 6 s lifetime | Unlocked at 90 s, weight 0.14, `max_active` 10, scrap 5 |
| **Exploder** | 60 HP, 4.6 m/s, mass 1.1, radius 0.45, half-height 0.45 | `Detonate` 3.0 m trigger, 0.7 s fuse, 45 dmg, 4.5 m radius, 9 knockback | Unlocked at 150 s, weight 0.10, `max_active` 8, scrap 4 |

All values are authored in `game/assets/config/enemies/enemies.ron`; the table documents the shipped roster, not a second source of truth.

---

## 3. AI Architecture & Horde Performance

### 3.1 Per-Zombie Finite State Machine (FSM)
```
        aggro radius                  target retained
[Idle] ─────────────▶ [Engage] ─────────────▶ [Engage / Attack]
   ▲                     │                          │
   │  alert elapsed      │  deaggro radius +        │ knockback > threshold
   │                     ▼  deaggro duration        ▼
   └──────────────── [Alert] ◀── stagger elapsed ─ [Stagger]
```
- **States:** `Idle`, `Alert`, `Engage`, `Stagger`, stored on `EnemyBrain` with the current target, last known target position, alert timer, stagger timer, and unseen duration.
- **Aggro:** an `Idle` enemy enters `Engage` and records the survivor as target when the survivor is within `aggro_radius`.
- **Deaggro:** an `Engage` enemy keeps updating its target while the survivor is within `deaggro_radius`; after the survivor has been outside that radius for `deaggro_secs` it enters `Alert` and remembers the last known position.
- **Alert:** steering follows the remembered position. Re-entering `aggro_radius` returns to `Engage`; when `alert_secs` elapses the brain clears its target and returns to `Idle`.
- **Stagger:** damage whose accumulated knockback velocity exceeds `stagger_threshold` enters `Stagger` for `stagger_secs`, cancels steering and wind-ups, then returns to `Engage` when a target exists, otherwise to `Alert`.
- **Damage forces engagement:** damage delivered by a survivor always engages the enemy against that source, regardless of distance.
- **Sensory Triggers (planned):** vision cone + **Noise Radius** (gunshots without suppressors, explosions, car alarms, extraction flares).
- **Tuning:** `aggro_radius` 18, `deaggro_radius` 26, `deaggro_secs` 2.5, `alert_secs` 4.0, `stagger_threshold` 4.5, `stagger_secs` 0.35, `acid_gravity` 9.81 in `enemies.ron`; hazard caps in `hazard.ron`.

### 3.1.1 Damage, Armor, and Death Contract
- **Damage message:** every damage delivery carries `kind` (`Ballistic`, `Explosive`, `Acid`, `Shock`) and `hit_dir` in addition to amount, knockback, and source. Weapons default to `Ballistic` and projectiles forward their authored kind and travel direction.
- **Armor:** an `Armor` component on an enemy holds `front_mult`, `back_mult`, `arc_deg`, and a `bypass` kind list. A hit travelling along `hit_dir` is frontal when `facing.dot(-hit_dir) >= cos(arc_deg / 2)`; frontal hits use `front_mult`, everything else uses `back_mult`, and any listed `bypass` kind applies full damage. A zero or non-finite `hit_dir` is treated as frontal so malformed data never grants the back bonus.
- **Death pipeline:** lethal damage marks the enemy `Dying`, emits `EnemyKilledMessage` once with the archetype's `scrap_value`, and keeps the entity alive for the rest of the frame. Death effects run in `DeathResolutionSet`; the cleanup pass then despawns `Dying` entities. `Dying` enemies are excluded from targeting, steering, melee, ranged attacks, detonation targets, and projectile collisions.
- **Ranged contract:** an archetype with a `Ranged` behavior holds a preferred distance (retreat inside `reach * 0.8`, hold through `reach * 1.15`, advance beyond), roots for the authored wind-up, and fires exactly one acid projectile when the wind-up completes with the target still inside `reach`. The cooldown starts on every completed wind-up, including a cancelled shot. The projectile ignores combatants, arcs under `acid_gravity`, and creates one hazard pool where it first touches world geometry, reaches the ground, or exhausts its lifetime. Pool radius and lifetime are authored on the `Ranged` behavior.
- **Detonation contract:** an archetype with a `Detonate` behavior arms an irreversible fuse when it is `Engage` and the survivor enters `trigger_radius`. An armed exploder roots in place. The blast resolves once per entity, either when the fuse elapses or when the entity is already `Dying`, and applies `Explosive` damage with `damage * (1 - distance / radius)` falloff plus outward horizontal knockback to the survivor and every non-`Dying` enemy. A proximity detonation still emits one `EnemyKilledMessage`; a death detonation relies on the kill already reported by the damage pipeline.
- **Ground hazards:** pools are run-scoped `HazardZone` entities under the `HazardPlugin`. `game/assets/config/hazard/hazard.ron` caps simultaneous zones (`max_active`) and owns the tick cadence (`tick_interval`). The first tick fires one interval after spawn, all later ticks follow that cadence, only the survivor takes `Acid` damage, and spawning past the cap retires the oldest zone first. Zones never damage enemies.
- **Scrap:** `EnemyKilledMessage.scrap` is the single source for pickup value; `ExtractionConfig` no longer holds per-kind scrap amounts.

### 3.1.2 Spawn Policy
- **Tiering:** an archetype only joins the spawn roll once `RunStats.elapsed_secs` reaches its `spawn.unlock_secs`.
- **Weighted roll:** eligible archetypes are selected proportionally to `spawn.weight`; a roster with no positive weight is rejected at load time.
- **Caps:** each archetype stops spawning at `spawn.max_active`, and the global active-enemy cap from escalation stops all spawning for that tick.
- **Placement:** enemies spawn on the arena edge at least `min_spawn_dist_from_survivor` away from every survivor, with the capsule center at `radius + half_height` so every archetype sits on the ground.
- **Hot reload:** stat and behavior values, spawn policy, and scrap values update from `enemies.ron` while running. Adding or removing a behavior on a live archetype logs a warning and applies on the next deploy.


### 3.2 Horde AI & Spatial Partitioning
Current implementation: separation is a pairwise `O(n²)` pass over active enemies, bounded by the escalation cap. The spatial grid below is the planned scaling step for large hordes.

To support large hordes smoothly in 3D:
- **Spatial Hash Grid:** Partition entities into uniform 2D/3D grid cells (e.g., 4m × 4m) rebuilt or updated every fixed tick.
  - Zombies query only neighbor cells for **separation steering** (avoid stacking into one point) and **local flanking**.
  - Turrets and AoE explosions use the exact same spatial grid for $O(1)$ nearby target queries instead of $O(N^2)$ scans.
- **Flow Field / Hybrid Pathfinding:**
  - Distant horde groups follow coarse region/nav-grid flow fields toward loud noise or the player.
  - Nearby zombies switch to direct steering + local obstacle avoidance (`bevy_rapier3d` raycasts/shape casts).

---

## 4. Multi-Layered Escalation System
The longer you stay in the 15–25 minute expedition, the more hostile the quarantine zone becomes across **four simultaneous layers**:

| Time Window (Example) | Threat Tier | Escalation Layers Active |
| :--- | :--- | :--- |
| **0:00 – 6:00** | Tier 1 (Calm) | Bright daylight; mostly Walkers & sparse Runners guarding buildings. |
| **6:00 – 12:00** | Tier 2 (Alert) | Tanks, Spitters, and Screamers enter spawn pools; first **Migrating Horde** crosses a main road. |
| **12:00 – 18:00** | Tier 3 (Danger) | **Dusk / Thick Fog** reduces visibility range; Exploders and Stalkers spawn inside buildings; spawn rates +50%. |
| **18:00 – 25:00+** | Tier 4 (Overrun) | **Nightfall** (flashlight & muzzle flashes light the dark); frequent roaming elites; relentless horde pressure until extraction. |

*Noise Escalation:* Making excessive loud noise (alarms, un-suppressed heavy gunfire) temporarily spikes local heat faster than the base timer.

*Data-Driven Tiers:* the shipped arena derives its spawn pool from `spawn.unlock_secs` per archetype (Walker and Runner from 0 s, Spitter from 90 s, Tank from 120 s, Exploder from 150 s) and shapes density with `spawn.weight`, per-archetype `max_active`, and the escalating global cap. The time windows above remain the design target for the full expedition mode.

---

## 5. High-Tension Extraction Mechanics
Extraction combines **all three** tension drivers so escaping feels earned without PvP:
1. **Timed Extraction Windows:** Extraction zones aren't permanently open. Each zone activates on a rotating schedule (e.g., Chopper arrives at North Pad from `14:00–16:00`, or South Gate opens for 90 seconds). Missing a window means surviving until the next window opens somewhere else.
2. **Signal Attracts Horde:** Activating the extraction beacon launches a flare / loud siren, triggering a large directional wave toward the extraction pad.
3. **Hold Position (30–60 Seconds):** You must remain inside or defend the extraction perimeter while the timer counts down. Leaving the zone pauses or resets progress, encouraging pre-placed turrets, mines, and traps around approaches.
