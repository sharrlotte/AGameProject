## MODIFIED Requirements

### Requirement: Block config is validated

Validation SHALL require unique block keys, a runtime feature for every feature config, a core block with group `"core"`, non-empty turret barrels, `range > 0`, `turn_speed >= 0`, `fire_rate > 0`, burst `amount > 0`, producer output `>= 0`, consumer rate `> 0`, storage capacity `> 0`, and a shape spec that bakes successfully: positive dimensions, rectangular `Rows` using only `#`/`.` with at least one filled cell and no empty border row or column, triangles whose rasterized bounding box equals their declared size, at least one cell, a single Manhattan-1 connected cell set, and a buildable triangle hitbox. Validation errors SHALL name the block.

#### Scenario: Empty turret barrels
- **WHEN** a turret declares an empty barrel list
- **THEN** loading reports a validation error naming the block

#### Scenario: Disconnected shape rejected
- **WHEN** a block's `Rows` describe two cell groups with no shared orthogonal edge
- **THEN** loading reports a validation error naming the block

#### Scenario: Ragged rows rejected
- **WHEN** a `Rows` shape has rows of differing lengths
- **THEN** loading reports a validation error naming the block

#### Scenario: Invalid row character rejected
- **WHEN** a `Rows` cell contains a character other than `#` or `.`
- **THEN** loading reports a validation error naming the block

#### Scenario: Triangle size mismatch rejected
- **WHEN** a triangle's rasterized bounding box is smaller than its declared size
- **THEN** loading reports a validation error naming the block
