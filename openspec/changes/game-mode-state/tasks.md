## 1. Define `GameMode` and `CampaignGame` States

- [ ] 1.1 Define `GameMode::{Skirmish, Campaign}` (`States`) and `CampaignGame` (`ComputedStates` over `(SceneState, GameMode)`) in `core/src/ui/state.rs` (re-exported via `core/src/resource/mod.rs`).
- [ ] 1.2 Register `.init_state::<GameMode>()` and `.add_computed_state::<CampaignGame>()` in `core/src/main.rs`.

## 2. Synchronize `WorldContext` with `GameMode`

- [ ] 2.1 Rename `WorldContext` variants (`QuickBattle` → `Skirmish`, `SaveSlot` → `Campaign`, keeping `Restoring`) in `core/src/save/world/component.rs` and remove `is_save_slot()`.
- [ ] 2.2 Update `WorldSavePlugin` handlers (`init_world_session`, `handle_create_slot`, `handle_load_slot`, `handle_continue_world`, `handle_quick_battle`, `reset_catalog_on_home`) to set `NextState<GameMode>` in lockstep with `WorldContext`.
- [ ] 2.3 Gate `tick_auto_save` (`core/src/save/world/mod.rs`) on `in_state(CampaignGame)` and `handle_save_world_event` on `State<GameMode> == Campaign`.

## 3. Migrate Quest Plugin and Pause/Skirmish Gating

- [ ] 3.1 Replace `save_slot_only` and `OnEnter(SceneState::Game)` HUD setup in `core/src/quest/mod.rs` and `hud.rs` with `in_state(CampaignGame)`, `OnEnter(CampaignGame)`, `OnExit(CampaignGame)` (clearing offer UI), and `DespawnOnExit(CampaignGame)`.
- [ ] 3.2 Remove all inline `if !context.is_save_slot()` guards and `Res<WorldContext>` parameters from `core/src/quest/system.rs`.
- [ ] 3.3 Update `core/src/scene/pause/mod.rs` (`on_ship_editor`, `on_home`, `setup_pause`) to read `Res<State<GameMode>>` instead of `session.is_save_slot()`.
- [ ] 3.4 Update `core/src/scene/game/skirmish/mod.rs` so skirmish enemy spawning runs cleanly according to `GameMode` and restore intent.

## 4. Verification and Docs

- [ ] 4.1 Add state-machine tests verifying `CampaignGame` computes `Some` only on `(SceneState::Game, GameMode::Campaign)`, Quest HUD spawns/despawns on `CampaignGame` enter/exit, and `GameMode::Campaign` persists across `Game -> ShipEditor`.
- [ ] 4.2 Update `docs/architecture.md`, `docs/quests.md`, `docs/scenes.md`, and `docs/save.md` (spec only, no notes) to document `GameMode` and `CampaignGame`.
