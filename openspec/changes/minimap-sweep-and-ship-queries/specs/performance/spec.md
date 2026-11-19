## ADDED Requirements

### Requirement: Minimap UI and AI scratch buffers avoid per-frame churn
The minimap SHALL NOT mutate marker `Node` layout properties on idle frames when neither the 1-second sweep snapshot nor active lost-contact flashes have changed. Minimap and AI tick systems (`sensor`, `board`, `readiness`, `scorer`, `booster`) SHALL reuse `Local` scratch collections instead of allocating fresh `HashMap` or `Vec` buffers every tick.

#### Scenario: Idle minimap frame performs no marker Node writes
- **WHEN** a frame runs between 1-second minimap sweeps with no active lost-contact flashes and no config change
- **THEN** `render_minimap` returns early without mutating `MinimapMarker` `Node` components

### Requirement: Enemy visibility sync only targets ships
`sync_enemy_visibility()` SHALL filter its query with `With<Ship>` so non-ship entities with `Team` and `Visibility` are not hidden by ship-contact visibility checks.

#### Scenario: Non-ship entity with Team is unaffected by sync_enemy_visibility
- **WHEN** an enemy-team non-ship entity (e.g., projectile or debris) has `Team` and `Visibility::Visible`
- **THEN** `sync_enemy_visibility()` does not set its `Visibility` to `Hidden`
