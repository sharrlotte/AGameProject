# Symbol Registry

## Requirements

### Requirement: Registry is keyed by interned symbols

Blocks, items, and palette groups SHALL be addressable by interned symbol ids
resolved at config load time; the compiled-in equivalent enum SHALL NOT exist.

#### Scenario: Fresh blocks load end-to-end
- **WHEN** the boot sequence reads `block.ron` with keyed defs (`hull`,
  `core`, …)
- **THEN** the `BlockRegistry` exposes `get("hull")` and returns the def,
  without any enum converting from string

#### Scenario: Adding a block does not touch Rust
- **WHEN** a designer adds an entry to `block.ron` and launches with no Rust
  recompile
- **THEN** the editor palette lists the new block and it can be placed

### Requirement: `.ron` block schema is keyed by id

`block.ron` SHALL be a mapping of id to definition, enforcing uniqueness by
construction and removing any redundant `block_type` field.

#### Scenario: Duplicate ids rejected at boot
- **WHEN** `block.ron` declares the same id twice
- **THEN** boot fails with a duplicate-id message naming the id

### Requirement: Embedded enums stay plain data

`ShootingProjectile`, `TargetingStrategy`, `ShootPattern` SHALL remain serde
enums embedded in turret configs; they are never interned or registered.

#### Scenario: A turret is defined without a registry roundtrip
- **WHEN** the turret component is inserted from config data
- **THEN** its projectile is a plain serde enum stored on the component and
  fired using that enum directly

### Requirement: Save format declares symbol-key breakage

The world-save path SHALL not attempt to load symbol-less (legacy enum-name)
schematics; it SHALL report a clear incompatibility error instead.

#### Scenario: Legacy schematic file fails at open
- **WHEN** the loader reads a JSON schematic written against the pre-symbol
  enum schema
- **THEN** it returns `Err` mentioning schema incompatibility rather than an
  empty or default ship
