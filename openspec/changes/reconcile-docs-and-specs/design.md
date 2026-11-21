## Context

Several recent changes (`rewrite-save-system`, `state-consistency-hardening`, `open-world-quests`, `shield-support`, `overdrive-field-display`, `world-chunk-streaming`) evolved the architecture rapidly, leaving contradictions both *between* specs and *inside* individual docs (`docs/save.md` asserts schema v3 on line 49 and schema v2 on line 191, and asserts `WorldSession`/`ActiveWorldStore` on line 151, `PendingWorld` on line 197, and `WorldContext` on line 201).

## Goals / Non-Goals

**Goals:**
- Make `/docs` and `openspec/specs/` a trustworthy, contradiction-free baseline reflecting the live codebase today.
- Follow `AGENTS.md` strictly: `/docs` contains concise normative specs only — no migration narratives, no "previously X was Y" notes, no obsolete type lists.
- Clean up the unreachable migration-fork notice in `core/src/save/world/system.rs:140-142`.

**Non-Goals:**
- Pre-documenting future changes from the other 12 proposals (each proposal's own `tasks.md` updates `/docs` when implemented).
- Fixing runtime bugs unrelated to documentation drift (captured in their dedicated proposals).

## Decisions

### 1. Reconcile Against Current Code First, Then Apply New Changes Cleanly
- **Choice**: Align all 4 drifted `/docs` files and 6 drifted `openspec/specs/` files to the current codebase baseline before applying the upcoming architectural proposals.
- **Rationale**: Delta specs (`MODIFIED Requirements`) in upcoming proposals rely on accurate baseline headers and non-contradictory specs in `openspec/specs/`. Reconciling the baseline first prevents cascading merge conflicts when archiving future changes.

### 2. Strict Spec-Only `/docs` Cleanup
- **Choice**:
  - `docs/architecture.md`: Update workspace tree (`resource/`, `quest/`, `control/`, `ui/`, `asset/`; remove `mission/`), update States table to the actual 2 states (`SceneState`, `ConfigState`) + `ModalStack`, update Typed Shared Resources (`WorldContext`, `DeployPayload.fleet`, `SessionClock`, `GameRng`, `QuestTracker`, `ShipCatalog`, `InspectedShip`), update Schedules/Plugins/Module map, and match `Cargo.toml` lint levels (`warn` for rust lints, `deny` for `clippy::absolute_paths`).
  - `docs/save.md`: Consolidate into one coherent v3 specification using `WorldContext`, `&self` `WorldStore` (12 methods), `quests` in `save_world` transaction, and strict `VersionMismatch` rejection; remove duplicate appended v2 sections.
  - `docs/scenes.md`: Update Home buttons (`Play` continues last save, `Worlds` opens `Modal::SlotPicker`), remove `SlotPickerState`/`PauseState`/`SettingState`/`DeployPayload.current_ship`, document `DeployPayload.fleet`, `CargoHudText`, `MinimapPlugin`, and `ModalStack` pause behavior accurately.
  - `docs/quests.md`: Align with `WorldContext` and actual skirmish/quest persistence scope.

## Risks / Trade-offs

- **Baseline changes slightly before upcoming proposals archive** → Keeping this reconciliation minimal and strictly factual (reflecting live code only) makes it safe to apply first as a fast, low-risk baseline commit with zero gameplay regressions.
