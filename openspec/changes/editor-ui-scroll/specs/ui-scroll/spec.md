## ADDED Requirements

### Requirement: Reusable vertical scroll area widget
The UI widget module SHALL provide a `ScrollPlugin` and `spawn_scroll_area` builder under `scene/widget/scroll/` that creates a `ScrollWrapper` containing a `ScrollViewport` (`Overflow::scroll_y()`, `flex_grow: 1.0`, `flex_basis: Val::Px(0.0)`, `min_height: Val::Px(0.0)`) and a sibling `ScrollbarTrack` with a `ScrollbarThumb`.

#### Scenario: Scroll area fits remaining column space
- **WHEN** `spawn_scroll_area` is spawned inside a `FlexDirection::Column` panel below fixed-height header elements
- **THEN** the scroll area occupies only the remaining vertical space without overflowing the bottom of the parent panel

### Requirement: Mouse-wheel scrolling is clamped
`send_scroll_events` and `on_scroll_handler` SHALL translate `MouseWheel` events over hovered UI entities into `Scroll` entity events that bubble to the nearest `ScrollViewport` and clamp `ScrollPosition.y` to `0.0..=max_offset.y.max(0.0)`.

#### Scenario: Scrolling past bounds
- **WHEN** the user scrolls past the top or bottom of a `ScrollViewport`
- **THEN** `ScrollPosition.y` clamps at `0.0` (top) or `max_offset.y` (bottom) without overscrolling

### Requirement: Visual scrollbar reflects scroll state and auto-hides
The `sync_scrollbars` system SHALL hide `ScrollbarTrack` when content height does not exceed viewport height, and otherwise size `ScrollbarThumb` proportionally (minimum 10% height) and position it according to `ScrollPosition.y / max_offset.y`.

#### Scenario: Content overflows viewport
- **WHEN** `ComputedNode.content_size().y` exceeds `ComputedNode.size().y`
- **THEN** `ScrollbarTrack` is visible and `ScrollbarThumb` height and top offset reflect the current scroll ratio

#### Scenario: Content fits within viewport
- **WHEN** `ComputedNode.content_size().y` is less than or equal to `ComputedNode.size().y`
- **THEN** `ScrollbarTrack` is hidden
