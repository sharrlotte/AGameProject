## MODIFIED Requirements

### Requirement: Shield Configuration and Block Feature
The engine SHALL support configuring shield features on blocks via `FeatureConfig::Shield` in block configuration files (`block.ron`). A shield configuration SHALL specify shape geometry (`Circle` with `radius` or `Plane` with `width`, `thickness`, and `offset`), `max_hp`, `cooldown` (seconds), `regen_rate` (hp per second), and optional power drain `demand`. Adding `Shield` to a non-disabled block entity SHALL trigger an `On<Add, Shield>` observer that spawns the `Barrier` child entity synchronously during command flush and records `Shield.barrier = Some(barrier)`.

#### Scenario: Block with Circle Shield spawns Shield and Barrier via observer
- **WHEN** a ship is instantiated with a block featuring a `Circle` shield and commands flush
- **THEN** a `Shield` component is inserted on the block entity and its `Barrier` child is spawned immediately by the `On<Add, Shield>` observer with configured radius, max health, cooldown, and regen rate

#### Scenario: Block with Plane Shield spawns Shield and Barrier via observer
- **WHEN** a ship is instantiated with a block featuring a `Plane` shield and commands flush
- **THEN** a `Shield` component is inserted on the block entity and its `Barrier` child is spawned immediately oriented to the block's placement transform and offset in front of the block
