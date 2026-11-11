# Combat, Movement & Weapons

## 1. Movement & Camera Feel
Combat responsiveness is the #1 priority. Movement and aiming are strictly decoupled.

### 1.1 Player Controls & Locomotion
- **8-Directional Movement:** `WASD` moves relative to world/camera ground axes with crisp acceleration and minimal slide.
- **Independent Mouse Aiming:** The character faces the 3D cursor raycast point on the aim plane, so strafing and backpedaling keep the weapon on target. The visual body turns to aim by default; sprinting above the movement threshold turns the body to horizontal velocity instead, even while aim is held.
- **Aim Stance:** Hold `RMB` to raise the aim stance. Firing or reloading raises it too, and a short hold timer keeps the arms up between semi-auto shots. Releasing every trigger fades the stance down after the hold expires.
- **Sprinting & Stamina:**
  - Holding `Shift` boosts speed (~1.45× walk speed) while draining a `Stamina` pool.
  - Sprinting prevents ADS/accurate fire (larger spread penalty or hip-fire only) to create a tactical choice: *reposition fast vs. stand and shoot accurately*.
  - Exhaustion (0 stamina) adds a short recovery delay before sprinting again.
- **Camera System:**
  - Angled top-down 3D perspective (e.g., 55°–65° pitch) with smooth damped follow.
  - **Aim-lead offset:** Camera subtly shifts toward the mouse cursor (10–20% of cursor distance from center) so the player can see further in the direction they are aiming.
  - **Dynamic screen shake:** Trauma-based micro-shake on heavy shots, explosions, and taking hits.

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
  - Visual reload progress bar near the reticle so the player never has to look away from combat. The reload animation is driven by the simulation timer, so reload-speed upgrades stay in sync with the clip.
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
