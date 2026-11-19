## Why

Currently, game persistence is fragmented across `saves/campaign.ron`, `saves/battle.ron`, `fleet.ron`, and `designs/*.ron`, which was designed around a single linear campaign rather than an open-world game with multiple save slots. Consolidating every playthrough's state (metadata, active world state, per-slot ship designs, and future tables for quests and chunks) into a single self-contained SQLite file per save slot makes creating, listing, copying, backing up, and deleting save slots trivial while keeping saves atomic and crash-safe.

## What Changes

- **BREAKING**: Replace `saves/battle.ron`, `saves/campaign.ron`, and global `fleet.ron` with **one self-contained SQLite file per save slot** under `<data_dir>/saves/<slot_id>.db` (WAL mode, `PRAGMA user_version` schema versioning, `rusqlite` bundled, `postcard` + `lz4_flex` payload encoding).
- Introduce a backend-agnostic `WorldStore` trait with `SqliteWorldStore` (production single-file DB) and `InMemoryWorldStore` (stateless `QuickBattle` sandbox and unit tests).
- Store per-slot ship designs inside each save slot's `designs` table (seeded from starter designs on New World creation) and route Ship Editor reads/writes through `DesignCatalogSource::ActiveWorldDb` when opened from a live save slot, while keeping a separate `HomeSandbox` catalog for Home's Ship Editor and `QuickBattle`.
- Add a Save Slot Picker UI launched from `Home` (`Play` → list slots with name, play time, last saved timestamp; actions: `Continue` / `Load`, `New World`, `Delete` with confirmation). Keep `QuickBattle` on Home as an unsaved in-memory sandbox.
- Remove the Main Menu Settings `Reset Mission / Campaign` section (replaced by per-slot Delete in the Save Slot Picker).
- Update `/docs/save.md`, `/docs/ui.md`, and `/docs/architecture.md`.

## Capabilities

### New Capabilities
- `world-save-slots`: Single-file SQLite save slot storage (`WorldStore` trait, `SqliteWorldStore`, `InMemoryWorldStore`), per-slot `designs` table, atomic single-transaction world saves, and Home Save Slot Picker UI.

### Modified Capabilities
- `world-save`: Transition from single `battle.ron` + `designs/*.ron` to single-file SQLite per save slot (`saves/<slot_id>.db`) with per-slot designs and Home sandbox isolation.
- `campaign-progression`: Replace single-save `campaign.ron` / `battle.ron` and Main Menu reset with multi-slot save management while preserving stateless `QuickBattle` and automatic saving on interval / exit.

## Impact

- **Dependencies**: Add `rusqlite` (`bundled`), `postcard`, and `lz4_flex` to `core/Cargo.toml`.
- **Systems & Modules**:
  - New `core/src/save/world/` module (`mod.rs`, `component.rs`, `store.rs`, `sqlite.rs`, `system.rs`) replacing `core/src/save/battle/`, `core/src/save/campaign/`, and `core/src/save/fleet/`.
  - `core/src/save/design/`: support `DesignCatalogSource::{ActiveWorldDb, HomeSandbox}` so Ship Editor works seamlessly both inside a save slot and from Home.
  - `core/src/scene/home/`: replace single `Play`/`Continue` with Save Slot Picker modal/view while retaining `QuickBattle`, `Ship Editor`, `Settings`, `Exit`.
  - `core/src/scene/pause/`: add `Ship Editor` entry when inside an active save slot, and flush/commit the active `WorldStore` on `Back to Home`.
  - `core/src/scene/setting/`: remove legacy `Reset Mission / Campaign` Data section.
- **Documentation**: Updates `docs/save.md`, `docs/ui.md`, and `docs/architecture.md`.
