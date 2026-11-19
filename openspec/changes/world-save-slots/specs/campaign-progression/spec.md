## MODIFIED Requirements

### Requirement: Quick Battle is stateless
`WorldSession::QuickBattle` SHALL run on `InMemoryWorldStore` and SHALL NOT create, read, overwrite, or delete any `<data_dir>/saves/*.db` file.

#### Scenario: Quick Battle leaves all save slots untouched
- **WHEN** the player launches `Quick Battle`, fights, pauses to Home, or exits
- **THEN** no file in `<data_dir>/saves/` is created, modified, or deleted

## REMOVED Requirements

### Requirement: Campaign progression is persisted separately from active battle state
**Reason**: Linear campaign progression (`campaign.ron` + `battle.ron`) is replaced by self-contained SQLite save slots (`world-save-slots`) and open-world quests (`open-world-quests`).
**Migration**: Use `<data_dir>/saves/<slot_id>.db` via `WorldStore`.

### Requirement: Campaign battles auto-save automatically
**Reason**: Replaced by save-slot auto-save inside `world-save-slots` (`WorldSession::SaveSlot`).
**Migration**: Auto-save commits to the active `SqliteWorldStore` on `auto_save_interval_secs` and on Pause `Back to Home`.

### Requirement: Home menu adapts Play vs Continue to active battle save
**Reason**: Single-save Play/Continue toggle is replaced by the multi-slot Save Slot Picker under `Play`.
**Migration**: Use the Save Slot Picker UI in `world-save-slots`.

### Requirement: Main Menu Settings includes a Data reset section
**Reason**: Global campaign reset is superseded by per-slot `Delete` with 2-step confirmation inside the Save Slot Picker.
**Migration**: Delete individual save slots from the Save Slot Picker.
