# Hand-Crafted World & Maps

## 1. Design Specification
Expedition maps are authored as 3D scenes in Blender and exported to glTF/GLB for direct loading in Bevy. Procedural generation is replaced with hand-crafted environments to ensure intentional tactical sightlines, cover placement, choke points, and environmental storytelling.

---

## 2. glTF Node & Marker Conventions

The map loader parses the glTF scene hierarchy on load and binds entities to simulation and presentation components based on object names and prefixes:

| Object / Empty Prefix | Function | Runtime Component / Behavior |
| :--- | :--- | :--- |
| `Spawn_Player` | Initial survivor insertion point and yaw | `Transform`, defines spawn position in `Scene::Arena` |
| `Spawn_Enemy` / `Spawn_Enemy_{Kind}` | Spawner markers for horde escalation | `EnemySpawner`, bounds spawn candidates by distance |
| `Collider_*` | Static physical collision geometry | Low-poly mesh converted to static `Collider` (`bevy_rapier3d`), `GROUP_WORLD` collision group; visual mesh hidden or invisible |
| `Visual_*` or default meshes | Pure visual environment geometry | Rendered with PBR materials, no physics colliders |
| `Trigger_Interior_{id}` | Building interior trigger volume | Sensor collider; entering triggers occlusion fade on associated roof meshes |
| `Roof_{id}` | Building roof and upper walls | Visual mesh linked to `Trigger_Interior_{id}`; rendered opaque normally, hidden/faded when survivor is inside |
| `Loot_{tier}` | Scavengeable container location | Interactable loot container (`Tier1`, `Tier2`, `Tier3`) |
| `Extract_Zone` | Expedition extraction pad | Sensor volume driving `ExtractionZone` countdown and holdout defense |

---

## 3. Physics & Static Collision Pipeline

1. **Dedicated Collision Meshes:**
   - Visual meshes NEVER generate automatic trimesh colliders.
   - All collisions are explicitly modeled as low-poly hull meshes in Blender prefixed with `Collider_` (e.g. `Collider_Wall_Brick`, `Collider_Car_Wreck`).
   - The scene loader extracts all `Collider_*` meshes, inserts Rapier static colliders (`Collider::convex_hull` or `Collider::trimesh`), attaches `Obstacle` markers, and strips render components so collision geometry is not drawn.

2. **Height & Cover Classification:**
   - **Low Cover (0.8m – 1.2m):** Waist-high concrete barriers, sandbags, car hoods. Blocks horizontal enemy movement; allows bullets to pass overhead.
   - **Full Obstacles (>2.0m):** Building walls, security fences, tall containers. Blocks line-of-sight, projectiles, and pathfinding.

---

## 4. Top-Down 3D Visibility & Interior Cutaways

Because the simulation uses full 3D from an angled top-down camera (55°–65° pitch):

1. **Trigger Volume Detection:**
   - Each building interior is bounded by a convex sensor volume named `Trigger_Interior_{id}`.
   - When the survivor entity overlaps this sensor, an `InteriorEnteredMessage { interior_id }` is emitted.
   - When the survivor exits, `InteriorExitedMessage { interior_id }` is emitted.

2. **Roof & Wall Occlusion:**
   - Meshes prefixed with `Roof_{id}` or `UpperWall_{id}` match the interior ID.
   - Inside the building: associated roof and upper-wall meshes are set to `Visibility::Hidden` (or material alpha faded), rendering the room fully readable from the top-down angle.
   - Outside the building: roofs render fully opaque with directional shadows cast on surrounding roads.

---

## 5. District 01 Specification: "Suburbs & Gas Station"

The initial hand-crafted vertical-slice map authors a self-contained quarantine district:

- **Dimensions:** 120m × 120m playable perimeter bounded by highway debris and concrete quarantine barricades.
- **Key Zones:**
  - **Insertion Zone (South-West):** Overgrown residential street, light initial Walker density.
  - **Gas Station & Convenience Store (Center):** Scavengeable interior with pharmacy counter, broken glass doors, roof cutaway trigger, and tight choke points. High Runner and Spitter presence.
  - **Auto Repair Bay (East):** Industrial warehouse with shipping crates, scrap containers, and Tank spawn area.
  - **Evacuation Helipad (North):** Elevated concrete pad, defensible sandbag perimeter, extraction beacon and final siege trigger.
