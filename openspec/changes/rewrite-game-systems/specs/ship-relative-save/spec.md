## REMOVED Requirements

### Requirement: Each ship saves as its own RON file

**Reason**: `WorldSave` embeds every ship in one slot file; per-ship files and their indirection are removed.

**Migration**: Use `world-save` slot files. Old `ships/*.ron` files are ignored and left on disk.

### Requirement: World stamp is a separate `world.ron`

**Reason**: The world snapshot is one self-contained slot file.

**Migration**: Use `world-save`. Old `world.ron` is ignored.

### Requirement: Save format is versioned and strict at read

**Reason**: Retained behavior moved to `world-save` with `SAVE_SCHEMA_VERSION = 2`.

**Migration**: Use the new version check; v1 saves are rejected with an error naming the versions.

### Requirement: Effect-like entities do not leak into the schema

**Reason**: Retained behavior moved to `world-save`.

**Migration**: No change; transients are still excluded from the schema and cleared on load.
