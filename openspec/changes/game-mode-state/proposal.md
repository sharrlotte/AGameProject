## Why

Campaign/save-slot gating is currently scattered across 9 manual `context.is_save_slot()` checks (`quest/mod.rs`, `quest/system.rs` ×3, `save/world/system.rs` ×2, `scene/pause/mod.rs` ×3) using a resource lookup (`WorldContext`). Meanwhile, `quest/hud.rs` is NOT gated at all — so the `"QUESTS"` HUD panel spawns even during Quick Battle — and `is_save_slot()` returns `true` during `WorldContext::Restoring`, blurring mode semantics. Additionally, the internal terminology (`Sandbox` / `QuickBattle` vs `SaveSlot`) mixes persistence mechanics with player-facing game modes.

## What Changes

- Introduce a top-level Bevy 0.20 `States` enum `GameMode::{Skirmish, Campaign}` (default `Skirmish`) renaming `Sandbox`/`QuickBattle` → `Skirmish` and `SaveSlot` → `Campaign`.
- Introduce a Bevy 0.20 `ComputedStates` struct `CampaignGame` derived from `(SceneState, GameMode)` that exists if and only if `(SceneState::Game, GameMode::Campaign)`.
- Migrate all quest simulation systems, auto-save (`tick_auto_save`), and Quest HUD lifecycle (`OnEnter(CampaignGame)`, `DespawnOnExit(CampaignGame)`, `OnExit(CampaignGame)` transient cleanup, `.run_if(in_state(CampaignGame))`) onto `CampaignGame`, removing `save_slot_only` and all manual `if !context.is_save_slot()` checks from `quest/` and `save/world/system.rs`.
- Migrate `Pause` (`scene/pause/mod.rs`) and `ShipEditor` mode checks onto `Res<State<GameMode>>` (since `GameMode::Campaign` persists across `Game ↔ ShipEditor` and resets to `GameMode::Skirmish` on `Home`/`StartQuickBattleEvent`).
- Align `WorldContext` variant naming cleanly (`Skirmish { store }`, `Campaign { slot_id, path, store }`, `Restoring { slot_id, path, store }`) and keep `GameMode` transitions synchronized inside `WorldSavePlugin`.

## Capabilities

### New Capabilities
- `game-mode-state`: Top-level Bevy 0.20 `GameMode::{Skirmish, Campaign}` state machine and `CampaignGame` computed state governing campaign gameplay, Quest HUD scoping (`DespawnOnExit(CampaignGame)`), autosave, and Pause/Editor mode checks.

### Modified Capabilities
- `world-session-intent`: Align `WorldContext` variants (`Skirmish`, `Campaign`, `Restoring`) with `GameMode::{Skirmish, Campaign}` synchronized by `WorldSavePlugin`.
- `open-world-quests`: Gate Quest HUD (`OnEnter(CampaignGame)` + `DespawnOnExit(CampaignGame)`) and all quest systems on `in_state(CampaignGame)` instead of `context.is_save_slot()`, ensuring zero quest HUD or progression exists in `GameMode::Skirmish`.

## Non-Goals

- Changing Home UI button labels beyond optional consistency with `Skirmish` / `Campaign` (keep Quick Battle / Play / Worlds behavior intact).
- Refactoring `QuestTracker` resources or objective listeners (covered by `consolidate-quest-resources` and `event-driven-quest-listeners`).

## Impact

- `core/src/ui/state.rs` (or `core/src/resource/mod.rs`): `GameMode` and `CampaignGame`.
- `core/src/main.rs` (`init_state::<GameMode>()`, `add_computed_state::<CampaignGame>()`).
- `core/src/quest/` (`mod.rs`, `system.rs`, `hud.rs`).
- `core/src/save/world/` (`component.rs`, `system.rs`, `mod.rs`).
- `core/src/scene/pause/mod.rs`, `core/src/scene/game/skirmish/mod.rs`.
- `docs/architecture.md`, `docs/quests.md`, `docs/scenes.md`, `docs/save.md` (spec-only updates).
