## 1. Reusable Scroll Widget (`scene/widget/scroll/`)

- [ ] 1.1 Create `core/src/scene/widget/scroll/component.rs` with `Scroll` (`EntityEvent`), `ScrollViewport`, `ScrollbarTrack`, and `ScrollbarThumb` components.
- [ ] 1.2 Create `core/src/scene/widget/scroll/system.rs` with `send_scroll_events`, clamped `on_scroll_handler`, and `sync_scrollbars` (auto-hide track + proportional thumb height/top).
- [ ] 1.3 Create `core/src/scene/widget/scroll/lib.rs` with `spawn_scroll_area` builder returning the inner `ScrollViewport` entity and spawning the sibling `ScrollbarTrack`/`ScrollbarThumb`.
- [ ] 1.4 Create `core/src/scene/widget/scroll/mod.rs` defining `ScrollPlugin` and re-exporting `scroll` types/helpers, wire it into `core/src/scene/widget/mod.rs`, and register `ScrollPlugin` in `core/src/main.rs` (or `WidgetPlugin`).

## 2. Editor UI Integration (`scene/editor/`)

- [ ] 2.1 Remove `Scroll`, `send_scroll_events`, and `on_scroll_handler` from `core/src/scene/editor/mod.rs` and `core/src/scene/editor/input.rs`.
- [ ] 2.2 Add `ShipListContent` marker in `core/src/scene/editor/mod.rs` and update `core/src/scene/editor/ui.rs` so `AiPanel`, `BlockListPanel`, and `ShipListPanel` use `spawn_scroll_area`, with `flex_shrink: 0.0` on `BlockSelectButton` and `AiPresetButton`.
- [ ] 2.3 Update `core/src/scene/editor/ship_list.rs` to query `ShipListContent` when clearing/populating ship rows and set `flex_shrink: 0.0` on each ship row.

## 3. Documentation & Verification

- [ ] 3.1 Update `docs/ui.md` to specify the reusable scroll widget and editor left-panel scrolling behavior.
- [ ] 3.2 Run `cargo check` (and `cargo dev` + screenshot/BRP check if applicable) to verify compilation and scrolling behavior in `SceneState::ShipEditor`.
