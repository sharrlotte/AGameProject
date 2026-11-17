# Ship Relative Save

## Requirements

### Requirement: Each ship saves as its own RON file

Saves SHALL produce one `ships/<id>.ron` per ship containing blocks with
grid-relative positions, turret state, pattern state, health, and team.

#### Scenario: Save round-trip preserves block state
- **WHEN** a save is taken mid-game and then loaded via the pause menu
- **THEN** every block exists at the same grid position with the same
  health, pattern state, and team, and no UUIDs appear in the file

### Requirement: World stamp is a separate `world.ron`

The world save SHALL contain world-level stamp data (teams, seed/rng, times),
not per-ship data.

#### Scenario: World load repopulates from ships
- **WHEN** a save is loaded from the pause menu
- **THEN** each `world.ron` references its ships (or ships directory is read
  as a side effect) and `spawn_ship_from_save` recreates entities

### Requirement: Save format is versioned and strict at read

Both files SHALL carry `version: <integer>` at the top; unknown versions
SHALL be rejected loudly with a message that names the version found.

#### Scenario: Future schema reads old break loudly
- **WHEN** `read_version()` sees a `version` other than the current
  `WORLD_SCHEMA_VERSION`
- **THEN** loading returns an error naming the expected and found versions
  instead of silently producing a broken world

### Requirement: Effect-like entities do not leak into the schema

The world save SHALL NOT serialize missiles, lasers, drops, or damage text;
they cannot appear in a freshly loaded world.

#### Scenario: Loaded world is clean of effects
- **WHEN** a world save is loaded
- **THEN** no missile, laser, drop, or damage text entity exists in the
  world, regardless of what was in flight at save time
