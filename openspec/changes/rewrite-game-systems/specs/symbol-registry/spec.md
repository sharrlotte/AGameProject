## REMOVED Requirements

### Requirement: Registry is keyed by interned symbols

**Reason**: `lasso` interners provided no deduplication because the interned key was discarded; block and item keys are plain `String` values now.

**Migration**: Use `BlockRegistry::get(&str)` and `String` fields. Config authors are unaffected.

### Requirement: `.ron` block schema is keyed by id

**Reason**: The keyed RON requirement is retained, but this capability is being removed as a unit because it exists to describe symbol interning.

**Migration**: Restated under `code-foundation`; `block.ron` remains a keyed mapping and duplicate ids still fail validation.

### Requirement: Embedded enums stay plain data

**Reason**: Retained behavior, moved to `code-foundation`.

**Migration**: No change for config authors; turret enums remain plain serde data.

### Requirement: Save format declares symbol-key breakage

**Reason**: The save format is a hard cutover with strict versioning, so a symbol-specific breakage path no longer exists.

**Migration**: Use `world-save` version rejection; old JSON and v1 files are ignored.
