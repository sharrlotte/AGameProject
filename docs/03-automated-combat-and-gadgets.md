# Automated Combat & Deployables

## 1. Role in Combat
Deployables turn your solo survivor into a tactical engineer. They should **reward planning and map reading** (holding hallways, covering your flank while looting, preparing extraction perimeters) without playing the entire game for you.

- **Deployment Constraints:** Carried in gadget slots with limited charges/weight per expedition (resupplied/crafted at Base or occasionally scavenged).
- **3D Placement Preview:** Holding a deployable shows a ghost preview on the ground with range ring, firing arc (if directional), and green/red validity check via `bevy_rapier3d` shape cast.

---

## 2. Deployable Equipment Roster

### 2.1 Stationary Turrets

| Equipment | Targeting & Combat Behavior | Resource Limit & Trade-off |
| :--- | :--- | :--- |
| **Gun Turret** | 360° or wide-cone fast-tracking light turret. Prioritizes closest unarmored threats (Walkers, Runners, Stalkers). | Limited internal ammo magazine (can be reloaded manually with player ammo or salvaged when packed up). Weak vs. Tanks. |
| **Heavy Turret** | Slower turn speed, high-caliber piercing rounds with knockback. Prioritizes highest-HP / Elite targets (Tanks, Screamers). | Slower setup time, heavier inventory weight, smaller ammo reserve, loud firing noise. |

**Target Acquisition Pipeline:**
1. Query spatial grid within `turret.range`.
2. Filter targets with `bevy_rapier3d` line-of-sight raycast (turrets cannot shoot through solid walls).
3. Score candidates by turret priority mode (`Closest`, `HighestHP`, `Fastest`) + small hysteresis bonus to current target so it doesn't jitter between two zombies.

### 2.2 Area-Denial Traps

| Equipment | Trigger & Effect | Tactical Use Case |
| :--- | :--- | :--- |
| **Electric Trap** | Floor pad/pylon; discharges when enemies step in radius, applying **Stun/Slow + shock DoT** for a few seconds, then enters a short recharge cooldown (has finite battery charges). | Place in doorways to freeze charging Runners or pause a Tank in front of your Heavy Turret. |
| **Proximity Mine** | Arms 1.5s after placement; detonates in a lethal 3D blast radius when any enemy enters trigger range. | Cover your rear while looting a building, or chain-detonate incoming hordes during an extraction holdout. |

### 2.3 Autonomous Drones

| Equipment | Behavior & AI | Upgrade / Customization Hooks |
| :--- | :--- | :--- |
| **Scout Drone** | Hovers above obstacle height; patrols ahead toward the cursor or unexplored rooms. **Highlights enemies, loot containers, and Stalkers** through walls/fog on your HUD/minimap. | Battery duration, scan radius, stealth (doesn't aggro zombies), container rarity tagging. |
| **Combat Drone** | Orbits/follows behind the player's shoulder at elevated height; automatically fires light bursts at enemies threatening the player (especially flankers). | Weapon module (SMG / taser dart / laser), target priority (Protect Owner vs. Hunt Specials), battery/ammo capacity. |

---

## 3. Synergy Examples
- **The Hospital Hallway Lock:** Drop an **Electric Trap** at the corridor choke + a **Gun Turret** behind a reception desk (waist-high cover). Walkers get zapped and shredded while you pick off Spitters.
- **Darkness Recon:** At Tier 3 Fog/Night, deploy the **Scout Drone** into an unlit Underground Lab wing to reveal two Stalkers waiting behind pillars before you step inside.
- **Extraction Perimeter:** Place 2 **Proximity Mines** on the blind alley behind the helipad, set up a **Heavy Turret** facing the main street, and rely on your **Combat Drone** to watch your back while reloading.
