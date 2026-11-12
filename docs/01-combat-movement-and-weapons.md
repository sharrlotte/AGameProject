# Combat, Movement & Weapons

## 1. Movement & Camera Feel
Combat responsiveness is the #1 priority. Movement and aiming are strictly decoupled.

### 1.1 Player Controls & Locomotion
- **8-Directional Movement:** `WASD` projects onto the camera yaw basis (world axes when no camera exists), with crisp acceleration and minimal slide.
- **Always-Aim Facing:** The character always faces the 3D cursor raycast point on the aim plane, including while sprinting, strafing, and backpedaling. There is no sprint-facing override and no residual spine twist; locomotion is chosen in aim space instead.
- **Aim Stance:** Hold `RMB` to raise the aim stance. Firing or reloading raises it too, and a short hold timer keeps the arms up between semi-auto shots. Releasing every trigger fades the stance down after the hold expires.
- **Sprinting & Stamina:**
  - Holding `Shift` boosts speed (~1.45× walk speed) while draining a `Stamina` pool.
  - Sprinting prevents ADS/accurate fire (larger spread penalty or hip-fire only) to create a tactical choice: *reposition fast vs. stand and shoot accurately*.
  - Exhaustion (0 stamina) adds a short recovery delay before sprinting again.
- **Camera System:**
  - Angled top-down 3D perspective (e.g., 55°–65° pitch) with smooth damped follow.
  - **Mouse-scroll zoom (`Ctrl + Scroll`):** Scales the camera offset along the fixed look ray between `zoom_min` and `zoom_max` with smooth exponential damping (`game/assets/config/camera.ron`), leaving plain `Scroll` available for weapon switching.
  - **Aim-lead offset:** Camera subtly shifts toward the mouse cursor (10–20% of cursor distance from center) so the player can see further in the direction they are aiming.
  - **Dynamic screen shake:** Trauma-based micro-shake on heavy shots, explosions, and taking hits.

### 1.2 Player Animation Stack

- **Aim-space locomotion:** The movement direction is projected onto the character's facing. A data-driven anchor list maps that angle to clips (`walk_fwd` 0°, `strafe_r` 90°, `walk_back` 180°, `strafe_l` -90° in `game/assets/config/player/animation.ron`). Each frame brackets the angle with the two nearest anchors and blends them, so diagonals already work with four cardinal clips and authored diagonals can be added without Rust changes.
- **Shared phase:** All anchors replay on one normalized phase clock. The dominant anchor seeks to the shared phase when it changes, so feet keep their cadence through direction changes. Playback speed scales with movement speed, with a higher sprint multiplier.
- **Layers:** Full-body base locomotion blend, an optional upper-body additive action layer (masked away from the lower body), and a death override. The always-aim yaw never overwrites locomotion.
- **Procedural weapon handling:** One chest-height anchor under `PlayerVisualRoot` holds a per-weapon presentation profile from `game/assets/config/player/weapons.ron` (model, sockets, relaxed/aim offsets, sway, recoil, reload, swap). Loose and aim poses are blended procedurally; sway, recoil, reload tilt/drop, and swap lower/raise are timelines driven by combat state rather than baked clips.
- **Dual-hand IK:** After animation evaluation, the right hand solves to the weapon `Grip` socket and the left to `Foregrip`, moving to `Magazine` during reload. Reach is clamped, elbows use pole hints, hands align to sockets, and optional shoulder settle is weighted and clamped.
- **Schedule:** In `PostUpdate`: drive → Bevy animation evaluation → weapon pose → arm IK → root yaw, all before transform propagation, only in `Playing`/`GameOver`.

---

## 2. Weapon Mechanics & Feel

### 2.1 Firing Models (Hybrid Hitscan & Projectile)
Different weapon classes use the model that feels best:
- **Fast Ballistic / Hitscan-like:** Pistols, SMGs, Rifles, Snipers can either use ultra-fast raycasts/tracers (clean hit registration) or high-speed physical projectiles with trail VFX.
- **Physical Projectiles:** Shotguns (pellet spread), Crossbows/Launchers, Turret rounds, and Spitter acid use `bevy_rapier3d` swept/kinematic projectiles or raycast-stepped projectiles to prevent tunneling at high speed.

### 2.2 Recoil, Spread & Reloading
- **Dynamic Spread (Bloom):** Firing increases a `CurrentSpread` angle; stopping fire decays it back to base weapon accuracy. Moving/sprinting multiplies base spread.
- **Recoil:** Applies visual kick to weapon/camera and temporarily widens the reticle; controlled by weapon `recoil_recovery`.
- **Reload State Machine:**
  - `Ready -> Reloading(timer) -> Ready`. Reloading continues while walking, strafing, and sprinting; movement never pauses or modifies the timer.
  - A weapon swap to a different slot cancels the reload with `ReloadCancelled(Swapped)`, applies no partial refill, and emits `WeaponSwapped`. Selecting the already-active slot keeps the reload running.
  - Player death cancels an active reload with `ReloadCancelled(Died)`; the death animation overrides every body layer.
  - Visual reload progress bar near the reticle so the player never has to look away from combat. The reload pose is a procedural timeline driven by the simulation timer, so reload-speed upgrades stay in sync with the animation.
- **Weapon Switching:** 2 Primary/Secondary slots + 1 Sidearm/Utility slot (`1`, `2`, `3`, or Scroll). Switch time varies by weapon weight.

### 2.3 Critical Hits, Knockback & Hit Feedback
- **Critical Hits:**
  - Base crit chance modified by weapon stats, attachments, and **aim precision** (e.g., standing still / low bloom, or hitting weakpoints like a Tank's exposed back).
  - Crits trigger distinct audio crack, larger floating damage color, and bonus stagger.
- **Enemy Knockback & Reactions:**
  - Every hit applies an impulse vector along the shot direction scaled by `weapon.knockback_force / enemy.mass`.
  - Heavy shotguns push Runners back mid-lunge; rifles stagger Walkers briefly; Tanks resist knockback unless hit by explosives or crits.
- **Juice / Feedback Checklist:**
  - Muzzle flash light + `bevy_hanabi` sparks/smoke.
  - Hit-stop / brief white flash shader on struck zombie meshes.
  - Directional blood/ichor decals or particles on impact.
  - Distinct impact audio for flesh vs. armor vs. concrete/metal.

---

## 3. Data-Driven Weapon & Attachment System

Weapons and mods are defined in external data files (RON/JSON) rather than hardcoded structs so balancing and adding content requires zero code changes.

### Example Weapon Definitions

| Weapon | Class | Fire Mode | Role & Feel |
| :--- | :--- | :--- | :--- |
| **Glock-19 / Sidearm** | Pistol | Semi-auto | Fast draw, tight moving accuracy, backup when primary runs dry. |
| **MP5-style SMG** | SMG | Full-auto | High fire rate, great mobility, struggles against armored Tanks. |
| **M4 Carbine** | Assault Rifle | Full-auto / Burst | Versatile workhorse; medium range, balanced recoil & penetration. |
| **Pump Shotgun** | Shotgun | Pump | Massive close-range burst + high knockback; stops Runners and pushes hordes. |
| **Marksman Rifle** | DMR | Semi-auto | High crit multiplier, pierces 1–2 Walkers in a line, deletes Screamers/Spitters at range. |
| **Flamethrower / Plasma** | Special | Continuous / AoE | Crowd control, area denial, melts swarms in tight corridors. |

### Attachment Slots & Examples
Weapons expose modular slots (e.g., `Optic`, `Muzzle`, `Magazine`, `Underbarrel`, `Core/AmmoMod`):
- **Suppressor (Muzzle):** Reduces gunshot noise radius (critical for not waking nearby buildings/hordes), slight velocity drop.
- **Extended Magazine:** +40% mag size, slightly slower reload.
- **Hollow-Point / Incendiary Core:** +25% flesh damage or burn DoT, reduced armor penetration.
- **Laser Sight (Underbarrel):** Tightens hip-fire spread while moving.

```
[Data Schema Concept]
WeaponDef {
  id: "ar_carbine_01",
  damage: 24.0,
  crit_chance: 0.12,
  crit_multiplier: 1.8,
  fire_rate_rpm: 650,
  mag_size: 30,
  reload_secs: 2.1,
  base_spread_deg: 2.5,
  max_spread_deg: 11.0,
  knockback: 3.5,
  noise_radius_m: 38.0,
  slots: [Muzzle, Optic, Magazine, Underbarrel]
}
```

---

## 4. Active Skills / Special Abilities
In addition to guns, players build around 1–2 active tactical abilities tied to equipment/progression:
- **Dash / Combat Roll:** Short invulnerability/burst reposition (costs stamina or cooldown) to escape Exploders or encirclement.
- **Concussion / Frag Grenade:** High-arc 3D throwable to clear rooms or stagger Tanks.
- **Overcharge Stim:** Temporary reload speed + fire rate boost during extraction sieges.
- **Pulse Scan:** Reveals enemies (including Stalkers) through walls briefly.
