# Triangle Hull

## Purpose

Triangle-shaped hull block definitions give starter ships pointed, swept silhouettes while keeping rendered art and baked collision geometry identical, and remain placeable in the ship editor.

## Requirements

### Requirement: Triangle hull definitions are available and collision-faithful

`block.ron` SHALL declare five hull-only triangle definitions: `wedge_1x1` (`Triangle(Right, (1,1))`), `cap_1x1` (`Triangle(Isosceles, (1,1))`), `plate_2x2` (`Triangle(Right, (2,2))`), `fin_1x2` (`Triangle(Isosceles, (1,2))`), and `spike_1x3` (`Triangle(Isosceles, (1,3))`). Each definition SHALL use its baked triangle polygon as its hitbox, with art alpha matching the true triangle, and SHALL be placeable in the ship editor with all four `Rot90` rotations.

#### Scenario: Triangle definitions load

- **WHEN** `config/base/block.ron` is loaded
- **THEN** the five triangle hull definitions are registered with baked `Hitbox::Polygon` hitboxes and no config error

#### Scenario: Empty half is not hittable

- **WHEN** a projectile passes through the empty half of a triangle hull block's bounding box
- **THEN** it does not collide with that block or with the ship's compound collider

#### Scenario: Editor exposes the pieces

- **WHEN** the ship editor builds its block list
- **THEN** all five triangle hull definitions appear and can be selected and rotated before placement

#### Scenario: Lighter explicit stats

- **WHEN** stats are derived from a ship containing triangle hull blocks
- **THEN** mass, health, cost and steel drops equal the per-size table: 1×1 = 6 / 600 / 3 / 6, 1×2 = 12 / 1200 / 6 / 12, 2×2 and 1×3 = 18 / 1800 / 9 / 18

### Requirement: Starter ships use triangle pieces for a ship silhouette

`Interceptor` SHALL have a crisp arrow silhouette and `Raider` SHALL have a broad swept wedge silhouette, both built with the triangle hull definitions. Both SHALL keep a single 3×3 core and the combat composition of three turrets, four small thrusters, two reactors, two batteries and one booster, and SHALL fit within Interceptor `x=-5..5, y=-5..6` and Raider `x=-6..6, y=-5..5`.

#### Scenario: Interceptor arrow

- **WHEN** the Interceptor starter design is spawned
- **THEN** its outline has a pointed nose formed by isosceles triangle pieces and straight diagonal edges formed by wedge and plate pieces

#### Scenario: Raider wide wedge

- **WHEN** the Raider starter design is spawned
- **THEN** its outline is a broad swept wedge with triangle pieces along the leading edges

#### Scenario: Starters remain structurally valid

- **WHEN** either starter design is validated
- **THEN** `validate_ship` reports exactly one core, orthogonally connected blocks, and no overlapping cells

#### Scenario: Starter power is feasible

- **WHEN** stats are computed for either starter design
- **THEN** generation plus booster draw capacity covers all active consumer demand

### Requirement: Triangle hull art is generated deterministically from the project art rules

`tools/gen_triangle_art.py` SHALL generate one RGBA PNG per triangle hull definition at the definition's base pixel size, drawing at half resolution and upscaling 2× with `NEAREST`, using the gunmetal base range `(38,42,50)`–`(86,94,106)` with amber accent `(214,158,46)`, diagonal top-left highlight and bottom-right shadow bevels, transparent pixels outside the true triangle, and no dark outer outline.

#### Scenario: PNG dimensions match the definition

- **WHEN** the generator is run
- **THEN** each PNG has width and height equal to the matching definition's base footprint size multiplied by `GRID_SIZE`

#### Scenario: Alpha matches the triangle

- **WHEN** a generated PNG is inspected
- **THEN** pixels inside the true triangle are opaque and pixels outside it are fully transparent

#### Scenario: Regeneration is stable

- **WHEN** the generator is run twice with no input changes
- **THEN** the two runs produce identical pixel data for every PNG
