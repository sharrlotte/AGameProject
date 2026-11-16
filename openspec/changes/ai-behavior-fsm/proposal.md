## Why

The AI FSM is one giant `match` over `AIShipBehavior` in `control_ship`, which
exhaustively matches every variant in the middle of an already-large function.
A per-behavior system-per-component layout makes adding behaviors local
(straightforward registration instead of editing the match), and makes every
behavior queryable by `With<Behavior::Kite>` for tooling/debug.

## What Changes

- Each `AIShipBehavior` variant becomes its own marker component carrying its
  parameters (`BehaviorChase { target: Entity }`, `BehaviorKite`,
  `BehaviorWander`, `BehaviorAttack`, `BehaviorGuard`, `BehaviorPatrol`,
  `BehaviorFollow`, `BehaviorKamikaze`, `BehaviorRegroup`, `BehaviorFlee`,
  `BehaviorIdle`).
- `AIShipController` is deleted or slimmed to a pure timer resource on the
  ship; behavior is inferred from the component, not a `controller.current()`
  enum query.
- `ship/system.rs` is split: one system per behavior transition, registered in
  the fixed update with explicit sets. The mega-match is gone; kiting/chasing
  logic moves into `ai/kite.rs`, `ai/chase.rs`, etc. (module split).
- Ship-U-HUD debug label reads the component directly instead of a `Debug`
  on the enum.
- No behavioural change is expected — same transitions, just expressed
  component-wise. **Not breaking.**

## Capabilities

### New Capabilities
- `ai-behavior-fsm`: AI ship behaviors are ECS components with one system per
  behavior; `AIShipController` deleted.

### Modified Capabilities
<!-- No existing specs. -->

## Impact

- Code: `content/ship/component.rs` (AIShipBehavior, AIShipController),
  `content/ship/system.rs` (split into `content/ship/ai/*.rs`),
  `content/ship/indicator.rs`, `scene/game.rs` (selection count display
  unchanged), `docs/ai.md`.
- No dependencies, no schema/format changes.
