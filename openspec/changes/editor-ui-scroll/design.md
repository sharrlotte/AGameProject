## Context

The ship editor's left panel (`core/src/scene/editor/ui.rs`) hosts three mutually exclusive tabs (`Blocks`, `Ships`, `AI`) below a fixed header ("Back to Home" + tab bar). Currently:
- `BlockListPanel` sets `height: Val::Percent(100.0)` and `Overflow::scroll_y()`, while its child `BlockSelectButton`s use default `flex_shrink: 1.0`. Taffy shrinks the buttons to fit instead of overflowing, so `max_offset.y == 0` and scrolling never engages.
- `ShipListPanel` and `AiPanel` have no `Overflow::scroll_y()`, and `ShipListPanel` has no height constraint.
- `Scroll`, `send_scroll_events`, and `on_scroll_handler` live inside `scene/editor/` (`mod.rs` / `input.rs`), are not state-gated, and do not clamp `ScrollPosition` to `[0.0, max_offset]`.
- There is no visual scrollbar track or thumb.

Project conventions (`AGENTS.md`) require Bevy 0.20.0 idioms, modular feature folders (`mod.rs`, `component.rs`, `system.rs`, `lib.rs`), no `unwrap()` or unsafe code, and recording UI specs in `docs/ui.md`.

## Goals / Non-Goals

**Goals:**
- Provide a reusable `scroll` module in `core/src/scene/widget/scroll/` (`ScrollPlugin` + `spawn_scroll_area`) that any scene can use.
- Support mouse-wheel vertical scrolling with clamped `ScrollPosition` (`0.0..=max_offset.y.max(0.0)`).
- Render a visual scrollbar track and thumb that reflects viewport-to-content ratio and scroll offset, and hides automatically when content fits without overflowing.
- Make all three editor left-panel tabs (`BlockListPanel`, `ShipListPanel`, `AiPanel`) scrollable with non-shrinking list items (`flex_shrink: 0.0`).

**Non-Goals:**
- Horizontal scrolling (`OverflowAxis::Scroll` on X) beyond preserving existing event delta handling.
- Drag-to-scroll on the scrollbar thumb (mouse-wheel scrolling is the primary interaction; thumb is a visual indicator).
- Scrollbar styling/theming loaded from `.ron` (use shared layout/color constants consistent with `scene/widget`).

## Decisions

### 1. Reusable `scene/widget/scroll/` module
- **Decision**: Move `Scroll`, `send_scroll_events`, and `on_scroll_handler` out of `scene/editor/` into `core/src/scene/widget/scroll/` split into `mod.rs`, `component.rs`, `system.rs`, and `lib.rs`.
- **Rationale**: Decouples UI scrolling from the editor so future scenes (menus, inventory, HUD lists) can reuse `spawn_scroll_area` and `ScrollPlugin` without duplicating scroll boilerplate.
- **Alternative**: Keep scroll systems in `scene/editor/`. Rejected because it violates separation of concerns and duplicates work when other scenes need scrollable lists.

### 2. Two-node wrapper + viewport hierarchy for tab visibility and scrollbar layout
- **Decision**: `spawn_scroll_area` spawns:
  - Outer `ScrollWrapper` (`FlexDirection::Row`, `flex_grow: 1.0`, `flex_basis: Val::Px(0.0)`, `min_height: Val::Px(0.0)`, caller-provided `Display` and marker component).
  - Inner `ScrollViewport` (`FlexDirection::Column`, `flex_grow: 1.0`, `min_height: Val::Px(0.0)`, `overflow: Overflow::scroll_y()`, `ScrollPosition::default()`), returned/passed to the caller's child-builder closure.
  - Sibling `ScrollbarTrack` (`width: Val::Px(6.0)`, `height: Val::Percent(100.0)`) containing `ScrollbarThumb` (`position_type: PositionType::Absolute`, `width: Val::Percent(100.0)`), linked to `ScrollViewport` via a `ScrollTarget(Entity)` component (or `ScrollbarOf(Entity)`).
- **Rationale**:
  - Placing `flex_grow: 1.0, flex_basis: Val::Px(0.0), min_height: Val::Px(0.0)` on the wrapper ensures the scroll area occupies *only* the remaining height below the editor's header/tab bar rather than overflowing the bottom of `left_panel`.
  - Putting the tab marker (`BlockListPanel`, `ShipListPanel`, `AiPanel`) on the `ScrollWrapper` lets `sync_tabs` toggle `Display::Flex` / `Display::None` for both the list and its scrollbar together.
  - `ship_list.rs` queries a dedicated `ShipListContent` marker (on the inner `ScrollViewport`) when despawning and repopulating ship rows, so the scrollbar sibling is never despawned.
- **Alternative**: Place `ScrollbarTrack` inside `ScrollViewport`. Rejected because children of `ScrollViewport` scroll with content and would require counter-offsetting every frame.

### 3. Thumb sizing and auto-hide
- **Decision**: A `sync_scrollbars` system queries `(&ComputedNode, &ScrollPosition)` on each `ScrollViewport` and updates its linked `ScrollbarTrack` (`Display::None` when `content_size.y <= size.y + 1.0`, `Display::Flex` otherwise) and `ScrollbarThumb` (`height: Val::Percent((visible / content * 100.0).clamp(10.0, 100.0))`, `top: Val::Percent(max_top_pct * (scroll_position.y / max_offset_y))`).
- **Rationale**: Avoids showing a redundant full-height thumb on short lists (e.g., `AiPanel` with few rules or `ShipListPanel` with 1–2 ships) and clamps minimum thumb height to 10% so very long lists remain clearly visible.

## Risks / Trade-offs

- **[Risk] Taffy shrinks list children instead of overflowing** → **Mitigation**: Set `flex_shrink: 0.0` on all fixed-height list items (`BlockSelectButton`, `ShipListPanel` rows, `AiPresetButton`).
- **[Risk] `send_scroll_events` runs in scenes without scrollable UI** → **Mitigation**: `send_scroll_events` exits immediately when `MouseWheel` is empty, so idle overhead is negligible; `on_scroll_handler` only mutates entities with `ScrollPosition` + `OverflowAxis::Scroll`.
- **[Risk] `ship_list.rs` despawns children of the wrong entity** → **Mitigation**: Use a distinct `ShipListContent` marker on the inner `ScrollViewport` while keeping `ShipListPanel` on the outer `ScrollWrapper` for `sync_tabs`.
