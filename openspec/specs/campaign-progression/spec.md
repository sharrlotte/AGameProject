# Campaign Progression

## Purpose

Single-save campaign progression with automatic battle snapshots, stateless Quick Battle isolation, and Main-Menu-only progress reset.

## Requirements

### Requirement: Quick Battle is stateless
`WorldSession::QuickBattle` SHALL run on `InMemoryWorldStore` and SHALL NOT create, read, overwrite, or delete any `<data_dir>/saves/*.db` file.

#### Scenario: Quick Battle leaves all save slots untouched
- **WHEN** the player launches `Quick Battle`, fights, pauses to Home, or exits
- **THEN** no file in `<data_dir>/saves/` is created, modified, or deleted
