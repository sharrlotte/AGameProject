# UI Buttons

## Purpose

All interactive UI buttons are built by one shared module using bubbling pointer events, so
clicks and hover feedback work uniformly over both labels and blank space. Actions live with
their domain rather than in a central registry.

## Requirements

### Requirement: One shared button builder

All interactive UI buttons SHALL be spawned through a single builder in `scene/widget.rs`. The
builder SHALL insert a domain-provided action component, a `ButtonVisuals` component holding
base/hover/press colors, and a `ButtonVisualState` component. Each action SHALL be a marker
component with its own observer function defined in the module that owns it; no central action
enum and no shared action context SHALL exist. Domain-specific `Changed<PickingInteraction>`
systems SHALL NOT remain.

#### Scenario: Adding a new button
- **WHEN** a new menu button is needed
- **THEN** its action component and observer are added in the owning domain module and passed to
  the shared builder, with no edit to `widget.rs` and no new central action variant

### Requirement: Clicks and hover work over labels

The builder SHALL attach the domain-provided `PointerClick` observer plus shared
`PointerOver`, `PointerOut`, `PointerPress`, and `PointerRelease` observers to each button
entity. Observers SHALL be attached per button and SHALL read the button from
`On::event().entity`, which is updated to the current ancestor as the event bubbles. Pointer
events bubbling from text-section children SHALL therefore activate and highlight the button.

#### Scenario: Clicking the label text
- **WHEN** the pointer presses and releases on the glyphs of a button's label
- **THEN** the `PointerClick` event bubbles to the button and the domain's click observer
  dispatches that button's action

#### Scenario: Clicking blank space
- **WHEN** the pointer presses and releases inside the button but outside its label
- **THEN** the same domain action is dispatched

#### Scenario: Hovering the label
- **WHEN** the pointer moves over the button's label text
- **THEN** `PointerOver` applies the hover color from `ButtonVisuals` to the button, and
  `PointerOut` restores the base color on exit

### Requirement: Visual state is data-driven

Dynamic appearance SHALL be expressed through `ButtonVisualState` (`Normal`, `Selected`,
`Disabled`). Domain systems SHALL set that state; one generic system SHALL derive the final
color from `ButtonVisuals` plus the state. No domain system SHALL write `BackgroundColor` for a
widget-built button directly.

#### Scenario: Active editor tab
- **WHEN** an editor tab becomes active
- **THEN** its button's `ButtonVisualState` is `Selected` and the derived color reflects that
  state

#### Scenario: Deploy already used
- **WHEN** the deployed ship matches the current editor design
- **THEN** the Deploy button's state is `Disabled` and its derived color is the disabled color

### Requirement: All button groups use the shared builder

All button groups SHALL use the shared builder: home, pause, save-slot, settings,
battle-overlay, ship-list HUD, and editor. No legacy `PickingInteraction` button handler SHALL
remain in those modules.

#### Scenario: Every group responds uniformly
- **WHEN** a user clicks any button in any of the groups
- **THEN** the click is handled by the shared observer regardless of whether the pointer was
  over text or blank space

### Requirement: Ship-list buttons are stable

The ship-list HUD SHALL update incrementally by the target ship entity. Button entities SHALL
survive a refresh when their ship is still present, and SHALL be removed only when the ship
disappears. Wholesale `despawn_children()` on every refresh SHALL NOT be used.

#### Scenario: Click during a refresh
- **WHEN** the 200 ms ship-list refresh runs between pointer press and release
- **THEN** the button entity still exists and the `PointerClick` event is delivered to its
  observer
