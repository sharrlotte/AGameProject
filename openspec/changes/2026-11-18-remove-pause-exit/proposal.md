## Why

The Pause menu exposes an **Exit** button that writes `AppExit::Success` immediately while also
queueing `PendingExitAction::Exit` to be flushed after the campaign auto-save completes. The
application can therefore close before `battle.ron` is captured. Application exit is already
available from `Home`, so the in-game path is redundant, races the save, and leaves an exit
variant in the deferred-exit plumbing that nothing else produces.

## What Changes

- Remove the **Exit** button, its `ExitButton` marker component, and its `on_exit` observer from
  `scene/pause.rs`.
- Reduce `PendingExitAction` to `{ None, Home }` and drop the `Exit` flush branch — and the
  now-unused `AppExit` writer parameter — from `save/io.rs`.
- Pause menu becomes **Resume**, **Settings**, **Back to Home**. `Home` keeps its **Exit** button
  and remains the only application quit surface.
- Campaign auto-save on Pause applies only to **Back to Home**.

## Capabilities

### Modified Capabilities
- `scene-lifecycle`: Pause no longer exposes an Exit action; application exit is available only
  from `Home`; `PendingExitAction` has no exit variant.
- `campaign-progression`: the Pause-triggered auto-save is `Back to Home` only.

## Impact

- `game/core/src/scene/pause.rs`: delete `ExitButton`, `on_exit`, and the `Exit` `spawn_button` call.
- `game/core/src/save/io.rs`: remove `PendingExitAction::Exit`, its flush arm, and the unused
  `MessageWriter<AppExit>` parameter.
- `docs/scenes.md`: Pause menu list and campaign auto-save trigger description.
- No new assets, config, or settings data. The Settings background is out of scope.

## Non-Goals

- Settings menu background or card — handled separately by the user.
- Removing `Home`'s **Exit** button or changing quit behavior from `Home`.
