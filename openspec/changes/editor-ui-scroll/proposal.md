## Why

The ship editor's left-panel lists (`Blocks`, `Ships`, `AI`) are too long to fit on screen and currently fail to scroll properly: `BlockListPanel` uses `height: 100%` with shrinkable items so content squishes instead of overflowing, `ShipListPanel` and `AiPanel` lack `Overflow::scroll_y()`, `on_scroll_handler` does not clamp `ScrollPosition`, and there is no visual scrollbar indicator.

## What Changes

- Extract a reusable `scroll` module under `core/src/scene/widget/scroll/` (`ScrollPlugin`, `Scroll`, `ScrollViewport`, `ScrollbarTrack`, `ScrollbarThumb`, `spawn_scroll_area`, mouse-wheel event dispatch, clamped scroll handler, and thumb sync system).
- Update all three editor left-panel tabs (`BlockListPanel`, `ShipListPanel`, `AiPanel`) to use `spawn_scroll_area` with `flex_shrink: 0.0` on list items so they overflow cleanly and show a visual scrollbar when content exceeds the viewport.
- Update `ship_list.rs` to populate ship rows into the inner `ScrollViewport` rather than the outer tab wrapper.
- Remove editor-local scroll types and systems (`Scroll`, `send_scroll_events`, `on_scroll_handler`) from `core/src/scene/editor/`.
- Record the scroll widget and editor UI scroll behavior in `docs/ui.md`.

## Capabilities

### New Capabilities
- `ui-scroll`: Reusable vertical scroll area widget with mouse-wheel scrolling, clamped `ScrollPosition`, and an auto-hiding visual scrollbar track and thumb.

### Modified Capabilities
- `editor-ui`: Left-panel tabs (`Blocks`, `Ships`, `AI`) support vertical scrolling with non-shrinking list items and visual scrollbars.

## Impact

- `core/src/scene/widget/mod.rs` and new `core/src/scene/widget/scroll/` module (`mod.rs`, `component.rs`, `system.rs`, `lib.rs`).
- `core/src/scene/editor/mod.rs`, `ui.rs`, `input.rs`, `ship_list.rs`, `ai_panel.rs`.
- `core/src/main.rs` (or wherever `WidgetPlugin` / `ScrollPlugin` is registered).
- `docs/ui.md`.
