## 1. Reconcile `/docs` Specifications (Spec-Only, No Notes)

- [ ] 1.1 Rewrite drifted sections of `docs/architecture.md`: workspace tree (`resource/`, `quest/`, `control/`, `ui/`, `asset/`; remove `mission/` and `AI_THINK_RATE_HZ`), States table (actual `SceneState`, `ConfigState`, and `ModalStack`), Typed Shared Resources (`WorldContext`, `DeployPayload.fleet`, `SessionClock`, `GameRng`, `QuestTracker`, `ShipCatalog`, `InspectedShip`), Schedules/Plugins/Module map, and workspace lint levels matching `Cargo.toml`.
- [ ] 1.2 Consolidate `docs/save.md` to eliminate all internal contradictions: schema version `3`, `WorldContext` (removing `WorldSession`, `ActiveWorldStore`, `PendingLoad`, `PendingWorld`), `&self` `WorldStore` trait (12 methods including `list_slots` and `load_quests`), `quests` in `save_world` transaction, strict `VersionMismatch` rejection (removing migration ladder mention), and accurate `setting.ron` path.
- [ ] 1.3 Update `docs/scenes.md`: Home buttons (`Play` continues last save, `Worlds` opens `Modal::SlotPicker`), remove `SlotPickerState`/`PauseState`/`SettingState`/`DeployPayload.current_ship`, document `DeployPayload.fleet`, `CargoHudText`, `MinimapPlugin`, editor overdrive fields, and `ModalStack` pause behavior.
- [ ] 1.4 Update `docs/quests.md` to align with `WorldContext` and current persistence/skirmish boundaries.

## 2. Clean Up Dead Migration Code Reference

- [ ] 2.1 Remove unreachable migration-fork notice check (`if active_slot_id != slot_id`) in `core/src/save/world/system.rs:140-142`.

## 3. Verification

- [ ] 3.1 Audit all 4 updated `/docs` files to confirm zero mentions of deleted types (`PendingLoad`, `PendingWorld`, `WorldSession`, `ActiveWorldStore`, `PauseState`, `SettingState`, `BattlePhase`, `SlotPickerState`, `DeployPayload.current_ship`, `mission/`) and zero non-spec historical notes.
- [ ] 3.2 Run `openspec validate` to confirm all 6 modified delta specs archive cleanly against `openspec/specs/`.
