# AI Behavior FSM

## Requirements

### Requirement: Each AI behavior is a distinct component

Each behavior in `AIShipBehavior` SHALL exist as a distinct marker or data
component, queryable independently, and each transition rule SHALL live in its
own system.

#### Scenario: Ship in Chase queries itself
- **WHEN** the AI debug overlay queries the current behavior
- **THEN** it reads `BehaviorChase { target }` and displays "Chase" without
  dereferencing a controller enum

### Requirement: Behavior implies exactly one marker at a time

Every AI-controlled ship SHALL carry exactly one behavior component at any
instant; transition systems `remove` the old variant before inserting the new.

#### Scenario: Wander → Chase transition
- **WHEN** a wandering ship detects an enemy in range
- **THEN** the transition system removes `BehaviorWander` and inserts
  `BehaviorChase { target }`, and no other behavior marker is present on the
  ship

### Requirement: AIShipController deleted

`AIShipController` SHALL not exist in the final code; behavior is inferred
from components and timing from a `BehaviorTimer` or the ship's own fixed
rate.

#### Scenario: Behaviour timer lives on ship
- **WHEN** the AI think-timer ticks and wants the next behaviour decision
- **THEN** the decision reads the marker component and any ship-local
  timer/state directly; no shared controller resource is consulted

### Requirement: docs/ai.md reflects the component FSM

The docs SHALL document the component-per-behavior layout, not the old enum.

#### Scenario: New contributor reads AI doc
- **WHEN** they open `docs/ai.md`
- **THEN** each behavior is listed as a component with its transition rules,
  and `AIShipController` is absent from the API reference
