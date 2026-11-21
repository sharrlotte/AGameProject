## ADDED Requirements

### Requirement: Block features implement one owning-module trait dispatched by macro
Each block feature struct (`Texture`, `TurretSpec`, `Generator`, `Battery`, `Consumer`, `Booster`, `Thruster`, `Storage`, `Shield`) SHALL implement the `BlockFeature` trait inside its own feature submodule (`content/block/feature/<name>/`). The `Feature` enum SHALL delegate `install`, `accumulate_stats`, `validate`, `icon`, and `texture` through a single declarative `feature_dispatch!` macro with no hand-written per-variant match arms in `Feature`, `BlockRegistry::stats_from_grid`, `BlockRegistry::validate_ship`, editor palette icon rendering, or the ship inspector diagram.

#### Scenario: Adding a new feature does not require editing stats or icon matches
- **WHEN** a new feature struct implements `BlockFeature` and is added to `feature_dispatch!`
- **THEN** `Feature::install`, `Feature::accumulate_stats`, `Feature::validate`, and `Feature::icon` dispatch to that implementation without editing `BlockRegistry`, editor icon rendering, or inspector diagram rendering

#### Scenario: Storage is a typed feature struct
- **WHEN** a block declares a storage capacity in `block.ron`
- **THEN** it maps to `Feature::Storage(Storage { capacity })` and contributes `capacity` via `Storage::accumulate_stats`

### Requirement: Multi-texture blocks render every texture layer without overwriting
When a block definition declares multiple `Texture` features, `spawn_block_entity` SHALL apply the first texture to the block's `Visual` entity (and its `z` layer to the block transform) and spawn every subsequent texture as a child `Sprite` of `Visual`, without `Feature::install` overwriting the primary `Sprite` with a later texture.

#### Scenario: Block with two textures keeps primary sprite and extra layer
- **WHEN** a block definition has two `Texture` features (`base` at `z = 10.0` and `decal` at `z = 12.0`) and is spawned via `spawn_block_entity`
- **THEN** `Visual` holds the `base` sprite, one child under `Visual` holds the `decal` sprite at `z = 12.0`, and `Visual`'s sprite is not replaced by `decal`

### Requirement: Feature types follow AGENTS.md naming and have no dead definitions
`TurretComponent` SHALL be renamed to `TurretSpec`, `ComponentPlugin` SHALL be renamed to `FeaturePlugin`, `Feature` SHALL NOT derive `Component`, unused `RecoilComponent` and `HeatEmitterComponent` SHALL be removed, and `BlockFeature::install` SHALL take a typed `SpawnCtx` rather than unused positional arguments.

#### Scenario: Dead types and Component suffixes are absent
- **WHEN** `core/src/content/block/` is inspected
- **THEN** `TurretComponent`, `ComponentPlugin`, `RecoilComponent`, and `HeatEmitterComponent` do not exist
