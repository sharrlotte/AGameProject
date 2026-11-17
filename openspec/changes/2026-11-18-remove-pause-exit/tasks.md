## 1. Pause Menu

- [ ] 1.1 Remove `ExitButton`, `on_exit`, and the `Exit` `spawn_button` call from `scene/pause.rs`.
- [ ] 1.2 Verify the remaining Pause buttons are **Resume**, **Settings**, **Back to Home** and
      that their observers are untouched.

## 2. Deferred Exit Plumbing

- [ ] 2.1 Remove the `Exit` variant from `PendingExitAction` in `save/io.rs`.
- [ ] 2.2 Remove the `Exit` match arm from `flush_pending_exit_action` and drop its now-unused
      `MessageWriter<AppExit>` parameter.
- [ ] 2.3 Confirm no references to `PendingExitAction::Exit` remain in `game/core/src` and that
      `Home`'s direct `AppExit::Success` write still works.

## 3. Specs and Documentation

- [ ] 3.1 Update `docs/scenes.md` Pause section: remove **Exit** from the menu list and describe
      campaign auto-save on **Back to Home** only.
- [ ] 3.2 Apply the OpenSpec deltas in `specs/scene-lifecycle` and `specs/campaign-progression`.
- [ ] 3.3 Validate the change with `openspec validate` and run a focused `cargo check -p core`
      (only if needed) before declaring implementation complete.
