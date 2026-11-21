## Context

Currently the codebase uses only 2 Bevy `States` (`SceneState`, `ConfigState`) and zero `ComputedStates` or `SubStates`. Because `WorldContext` is a `Resource`, systems cannot use `OnEnter`, `OnExit`, or `DespawnOnExit` for campaign-only entities or transient cleanup. As a result:
- `quest/mod.rs:65,78-80` adds a custom `save_slot_only` run condition, AND `quest/system.rs:136,229,273` repeats `if !context.is_save_slot()` inside system bodies.
- `quest/hud.rs:11-83` spawns `"QUESTS"` HUD unconditionally on `OnEnter(SceneState::Game)` — leaking an empty `"QUESTS / No active quests"` header into Quick Battle (`Skirmish`).
- `scene/pause/mod.rs:97,121,179` checks `session.is_save_slot()` directly, and `save/world/system.rs:51,510` checks `context.is_save_slot()`.

## Goals / Non-Goals

**Goals:**
- Represent game mode cleanly as a Bevy 0.20 `States` enum: `GameMode::{Skirmish, Campaign}`.
- Derive `CampaignGame` via `ComputedStates` from `(SceneState, GameMode)` so campaign-only gameplay systems and HUD entities gain full Bevy state lifecycle support (`in_state(CampaignGame)`, `OnEnter(CampaignGame)`, `OnExit(CampaignGame)`, `DespawnOnExit(CampaignGame)`).
- Keep `GameMode::Campaign` active across `SceneState::Game ↔ SceneState::ShipEditor` so Pause and Ship Editor always know whether the player is editing a campaign slot or a Home skirmish sandbox.
- Remove `save_slot_only` and all 9 scattered `is_save_slot()` checks.

**Non-Goals:**
- Splitting `SceneState::Game` into `GameCampaign` and `GameSkirmish` variants (duplicating every shared gameplay system).
- Replacing `ModalStack` with Bevy states.

## Decisions

### 1. Top-Level `GameMode` (`States`) + `CampaignGame` (`ComputedStates`)
- **Choice**:
  ```rust
  #[derive(States, Default, Clone, Copy, Eq, PartialEq, Hash, Debug, Reflect)]
  pub enum GameMode {
      #[default]
      Skirmish,
      Campaign,
  }

  #[derive(ComputedStates, Clone, Copy, Eq, PartialEq, Hash, Debug)]
  pub struct CampaignGame;

  impl ComputedStates for CampaignGame {
      type SourceStates = (SceneState, GameMode);

      fn compute(sources: (SceneState, GameMode)) -> Option<Self> {
          match sources {
              (SceneState::Game, GameMode::Campaign) => Some(Self),
              _ => None,
          }
      }
  }
  ```
- **Rationale**:
  - `GameMode` stays `Campaign` while visiting `SceneState::ShipEditor` from Pause, and resets to `GameMode::Skirmish` when returning to `SceneState::Home` or starting Quick Battle.
  - `CampaignGame` activates automatically when entering `SceneState::Game` in `GameMode::Campaign` and deactivates immediately on exiting `Game` or switching to `Skirmish`.
  - Quest HUD spawns in `OnEnter(CampaignGame)` tagged with `DespawnOnExit(CampaignGame)` — completely absent in `Skirmish`.
  - All quest simulation, HUD sync, and `tick_auto_save` systems use `.run_if(in_state(CampaignGame))` (or a `QuestSet` configured with `in_state(CampaignGame)`).
  - `OnExit(CampaignGame)` clears transient offer UI cleanly.
- **Alternatives Considered**:
  - *`SubStates` (`PlayMode`) scoped to `SceneState::Game`* — disappears when leaving `Game` for `ShipEditor`, forcing `Pause`/`ShipEditor` back onto `WorldContext`.
  - *`QuestSet` gated by `save_slot_only`* — provides no `OnEnter`/`OnExit`/`DespawnOnExit` lifecycle for Quest HUD or transient UI cleanup.

### 2. Single Synchronization Point Between `WorldContext` and `GameMode`
- **Choice**: Rename `WorldContext` variants to `Skirmish { store }`, `Campaign { slot_id, path, store }`, and `Restoring { slot_id, path, store }`. Every transition in `WorldSavePlugin` (`handle_create_slot`, `handle_load_slot`, `handle_continue_world`, `handle_quick_battle`, `reset_catalog_on_home`) updates both `WorldContext` and `NextState<GameMode>` in lockstep via one small helper `set_mode(&mut next_mode, GameMode::Campaign | GameMode::Skirmish)`.
- **Rationale**: Guarantees `WorldContext` and `State<GameMode>` never diverge. Unit tests verify both transitions and `CampaignGame` activation/deactivation.

## Risks / Trade-offs

- **Frame timing of `NextState<GameMode>` + `NextState<SceneState>`** → In Bevy 0.20, queueing `next_mode.set(GameMode::Campaign)` and `next_scene.set(SceneState::Game)` in the same `Update` frame resolves both in the next `StateTransition` schedule before `OnEnter(SceneState::Game)` and `OnEnter(CampaignGame)` run, so `CampaignGame` is already active when entering `Game`.
- **Unit tests running individual quest systems directly** → Unit tests no longer need a dummy `WorldContext` resource just to satisfy `if !context.is_save_slot()`; systems become pure and test-friendly, while state gating is tested directly with `StatesPlugin` + `GameMode` + `CampaignGame`.
