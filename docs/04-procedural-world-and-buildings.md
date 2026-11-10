# Procedural World & Building Generation

## 1. Design Goals
- **Deterministic Seeded Generation:** Every expedition uses a `u64` seed so layouts, loot, and spawns are reproducible (critical for bug reports, save/resume on exit, and future multiplayer sync).
- **Readable Spaces, Not Random Mazes:** Exterior districts connect logically via roads/highways, and building interiors are assembled from **handcrafted room templates** snapped along valid doorways so rooms feel like real places.
- **3D Chunked Streaming:** World geometry and colliders are partitioned into spatial chunks for fast culling, lighting, and horde AI partitioning.

---

## 2. The 7 Region / Biome Types

Each expedition generates a connected graph of 4–6 regions scaled to a **15–25 minute** run:

| Environment | Gameplay Purpose | Typical Threats & Terrain |
| :--- | :--- | :--- |
| **City Center** | Dense multi-room buildings, high-value loot, high enemy density. | Tight alleys, multi-story lobbies, broken glass/alarms, Screamers & Runners. |
| **Suburbs** | Residential houses, food, basic medical/crafting supplies. | Low-risk starting/transition zone; fences (vaultable/waist-high), garages, Walkers. |
| **Industrial District** | Machinery, scrap metal, electronics, fuel/turret parts. | Wide warehouse bays, shipping containers, forklifts, Tanks & Exploders. |
| **Hospital** | High-tier medical supplies, stims, bio-samples, tight corridors. | Maze-like wings, dark patient wards, Stalkers & Spitters. |
| **Military Facility** | Advanced weapons, armor plates, heavy ammo, tactical mods. | Sandbag perimeters, watchtowers, locked armory bunkers, armored Tanks. |
| **Underground Laboratory** | Rare research tech, blueprints, special mutated enemies. | Elevator/stair descent, power-gated security doors, narrow labs, Stalkers & Screamers. |
| **Highway** | Vehicle wrecks, military supply convoys, long sightlines. | Burned cars (3D cover), elevated overpass ramps, migrating hordes. |

---

## 3. Five-Stage Procedural Generation Pipeline

```
Stage 1: Macro Region Graph
  Generate connected region nodes (e.g., Suburbs ─ Highway ─ City ─ Hospital ─ Lab)
  Assign extraction points & difficulty gradient from spawn edge.
        │
        ▼
Stage 2: Road Network & Building Footprints
  Carve connecting roads/highways, place block parcels, outer walls, fences,
  and 3D height features (overpasses, stairs, courtyards).
        │
        ▼
Stage 3: Building Interiors (Handcrafted Room Templates)
  Assemble buildings by connecting typed room modules via door sockets.
        │
        ▼
Stage 4: Gameplay Population
  Place loot containers by room tag, enemy spawn nests, objectives, & cover props.
        │
        ▼
Stage 5: Environmental Storytelling & Random Events
  Add blood trails, barricades, flickering lights, destructible props, & event triggers.
```

### Stage 3 Deep Dive: Handcrafted Room Templates (Hospital Example)
Instead of pure BSP/noise rooms, each building archetype has a grammar graph and a library of pre-authored 3D room prefabs (`RoomTemplate`) with tagged doorway sockets (`North`, `South`, `East`, `West`, `StairUp/Down`):

```
                 ┌────────────────────┐
                 │ Restricted Lab     │ (Keycard / Power gated)
                 │ [High-Tier Loot]   │
                 └─────────▲──────────┘
                           │ Security Door
┌──────────────┐   ┌───────┴───────┐   ┌──────────────┐
│ Patient Ward │◀─▶│ Main Corridor │◀─▶│   Pharmacy   │
│ (4-6 beds)   │   │  (Choke Pt)   │   │ (Meds/Stims) │
└──────────────┘   └───────▲───────┘   └──────────────┘
                           │
        ┌──────────────────┼──────────────────┐
        ▼                  ▼                  ▼
┌──────────────┐   ┌───────────────┐   ┌──────────────┐
│ Emergency    │   │ Reception &   │   │   Storage    │
│ Dept (ER)    │   │ Waiting Room  │   │ (Scrap/Ammo) │
└──────────────┘   └───────▲───────┘   └──────────────┘
                           │ Street Entrance
```

- **Room Template Rules:**
  - *Reception* always connects to an exterior street/parking lot.
  - *Pharmacy* spawns behind a locked/reinforced counter or side corridor.
  - *Restricted Research Area* requires restoring local power or finding a keycard in the ER/Reception.
  - Every room variant includes pre-authored **3D cover nodes** (desks, gurneys, pillars) and **spawn markers** so AI never spawns stuck inside geometry.

---

## 4. Top-Down 3D Visibility & Interior Cutaways
Because the game uses **full 3D (`bevy_rapier3d`)** from an angled top-down camera:
- **Roof / Upper-Wall Occlusion:** When the player steps inside a building (or aims into a doorway), roofs and camera-facing tall walls smoothly fade out or clip above waist height so the interior stays 100% readable while exterior 3D roofs cast shadows around the block.
- **Line-of-Sight Fog of War:** Rooms you haven't entered (or scanned with a Scout Drone) remain shadowed/dimmed behind closed doors, preserving tension when opening doors into dark wards.
