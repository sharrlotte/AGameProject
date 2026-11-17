## Context

`scene/pause.rs` spawns four buttons: **Resume**, **Settings**, **Back to Home**, **Exit**.
`on_exit` currently sets `PauseState::Playing`, writes `SaveWorldEvent` and
`PendingExitAction::Exit` when leaving a `Campaign` battle, and also writes `AppExit::Success`
directly. `save/io.rs::flush_pending_exit_action` handles the `Exit` variant by writing
`AppExit::Success` after the save completes.

`PendingExitAction::Exit` has exactly one producer — that `on_exit` observer. `Home`'s
`ExitButton` writes `AppExit::Success` directly and does not use `PendingExitAction`.

## Goals / Non-Goals

**Goals:**
- Remove the Pause **Exit** button so quitting during a battle goes through
  **Back to Home → Exit**.
- Eliminate the duplicate/immediate app-exit write that can race the campaign auto-save.
- Leave no dead exit variant or unreachable flush branch behind.

**Non-Goals:**
- Any Settings menu background work.
- Removing or restyling `Home`'s **Exit** button.
- Changing the deferred save flow used by **Back to Home** (`PendingExitAction::Home`).

## Decisions

### 1. Remove only the Pause Exit button

- **Decision**: Delete `ExitButton`, `on_exit`, and the `Exit` button spawn from `pause.rs`; keep
  `Home`'s `ExitButton` unchanged.
- **Rationale**: Matches the request exactly and preserves a single, explicit quit surface on
  `Home`. Campaign progress is still captured when leaving via **Back to Home**.
- **Alternative considered**: Keep Exit but remove its immediate `AppExit::Success` write.
  Rejected — the request is to remove the button, not repair its behavior.

### 2. Prune `PendingExitAction::Exit` fully

- **Decision**: Remove the `Exit` enum variant, its `flush_pending_exit_action` match arm, and the
  now-unused `MessageWriter<AppExit>` parameter from that system.
- **Rationale**: The variant has no other producer; leaving it would be dead code and a second,
  unused exit code path. Removing it also removes the immediate-app-exit race.
- **Alternative considered**: Keep the variant as a future hook. Rejected — the project rules
  favour a single source of truth and no unreachable code.

### 3. Settings background is out of scope

- **Decision**: Do not add any card, image, color, or config for the Settings menu in this change.
- **Rationale**: The user is handling that separately.

## Risks / Trade-offs

- **[Risk] No direct quit from Game / ShipEditor** → **Back to Home** remains available from Pause
  and still triggers the campaign auto-save; quitting then takes one extra click on `Home`.
- **[Risk] `PendingExitAction` API change** → repository-wide search confirms the `Exit` variant is
  referenced only in `pause.rs` and `save/io.rs`; `Home` writes `AppExit` directly.
