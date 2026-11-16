## Context

Today AI behaviour lives in one 600-line `control_ship` function which:
  - rebuilds two `HashMap` indexes (`teams`, `enemies`) per frame,
  - matches on `AIShipBehavior` (10+ variants) in one match arm,
  - mixes selection policy with physics/integration and UI-adjacent code.

Separating each variant into its own component + system makes the behaviour
set easier to extend (new behavior = new module + plugin registration), makes
tooling trivial (`Query<&BehaviorChase>`), and removes the "add another match
arm in the middle of a 600-line function" hazard.

## Goals / Non-Goals

**Goals:**
- One marker component per behaviour variant; UI/AI/gizmos query them directly.
- One transition system per behaviour, registered in the fixed update.
- No behavior change: transition rules are ported one-for-one.
- `ai` module split from `ship` for file size.

**Non-Goals:**
- Better/different AI choices than v1. No tuning.
- Replacing the `Wander/Guard/Patrol` randomness sources (still `rand::rng()`).
- Removing the `ShipBehavior { Approach, GoAround }` intent components — those
  are already well-formed data-on-components; leaving as-is.

## Decisions

- **Components instead of enums.** One system per component takes
  `Query<(Entity, &mut Transform, &mut ExternalForce, &Behavior), ...>`,
  reads the context (nearby ships — cached index) and decides whether to
  stay, transition, or emit `ShipBehavior`. Rationale: behaviour-owned code
  locality, no central dispatch, no exhaustive-match burden.
- **Keep team/enemy indexes.** The `teams`/`enemies` HashSet rebuild per
  fixed tick is O(S·T) and is the same across behaviours; keep it as a
  shared resource `ShipIndex { teams: HashMap<u8, Vec<...>>, }` in this slice
  (rebuilding per behaviour would be worse). Future optimization can swap it
  for a spatial index without changing behaviour code.
- **`AIShipController` deleted** in favour of a `BehaviorTimer` component
  carrying only `Timer`; transitions read it in each behaviour system.
  Rationale: the field `last_behavior`/`initial_behavior`/`current` go away
  once behaviour is implied by the component set.
- **`ShipBehavior` (Approach / GoAround) intentionally kept** — it is already
  a small, data-rich component pair with two variants; breaking it up further
  adds structure without capability.

## Risks / Trade-offs

- [Risk] Transition order now depends on plugin registration order; bugs like
  "behaviour didn't react this frame" become order-sensitive. → Mitigation:
  document order in `ai::mod` and keep systems in one `in_set(...)` gate.
- [Risk] Entity count slightly larger due to separate marker components per
  behavior state — negligible (one component per ship). → none.
- [Risk] Debug HUD loses a single `controller.current()` read — must query the
  marker components instead. → straightforward replacement.
- [Risk] Dual-counting ships (two marker components present at once during
  transition). → Transition systems `remove::<X>().insert(Y)` in the same
  system run; queries use `With<Y>` not `Option<&X>` so a ship matches exactly
  one behaviour's filter.

## Migration Plan

1. Introduce `ai/` module with per-variant components first; port one behavior
   (`Idle`) as a template, then port the rest one at a time.
2. Delete `AIShipController`; switch HUD debug label to component query.
3. Docs sync (`docs/ai.md`).

Rollback: revert — no schema or persistent data is touched.
