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

---

## 3. AI Architecture & Horde Performance

### 3.1 Per-Zombie Finite State Machine (FSM)
```
        noise / sight                  in range
[Idle / Wander] ──────▶ [Alert / Investigate] ──────▶ [Engage / Attack]
       ▲                                                     │
       │ lost target (timeout)                               ▼
       └───────────────────────────────────────────── [Stagger / Disabled]
                                                       (knockback / electric trap)
```
- **Sensory Triggers:** Vision cone + **Noise Radius** (gunshots without suppressors, explosions, car alarms, extraction flares).

### 3.2 Horde AI & Spatial Partitioning
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

---

## 5. High-Tension Extraction Mechanics
Extraction combines **all three** tension drivers so escaping feels earned without PvP:
1. **Timed Extraction Windows:** Extraction zones aren't permanently open. Each zone activates on a rotating schedule (e.g., Chopper arrives at North Pad from `14:00–16:00`, or South Gate opens for 90 seconds). Missing a window means surviving until the next window opens somewhere else.
2. **Signal Attracts Horde:** Activating the extraction beacon launches a flare / loud siren, triggering a large directional wave toward the extraction pad.
3. **Hold Position (30–60 Seconds):** You must remain inside or defend the extraction perimeter while the timer counts down. Leaving the zone pauses or resets progress, encouraging pre-placed turrets, mines, and traps around approaches.
