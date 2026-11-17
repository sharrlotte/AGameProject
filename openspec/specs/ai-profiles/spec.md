# AI Profiles

## Purpose

Per-ship inline AI profiles of prioritized rules drive one runtime behavior enum, replacing marker-based behavior FSMs.

## Requirements
### Requirement: AI profiles are per-ship and inline

Every `ShipDesign` SHALL contain an inline `AiProfile { think_rate_hz, initial, rules }`; there SHALL be no global faction behavior config. Each rule SHALL contain priority, condition, behavior, and cooldown.

#### Scenario: Two ships, two profiles
- **WHEN** two ships are spawned from different designs
- **THEN** they run different rule sets without a shared resource

### Requirement: Rules are prioritized and cooldown-gated

At `think_rate_hz` in `FixedUpdate`, the AI SHALL evaluate rules in descending priority, apply the first matching condition, continue the current behavior when none match, and respect per-rule cooldowns to prevent thrash.

#### Scenario: Priority wins
- **WHEN** two rules match
- **THEN** the higher-priority behavior runs

#### Scenario: Cooldown allows fallback
- **WHEN** a rule is on cooldown
- **THEN** a lower-priority matching rule may run

#### Scenario: No matching rule
- **WHEN** no rule matches
- **THEN** the current behavior continues

### Requirement: All authored behaviors are reachable

The AI SHALL implement `Idle`, `Wander`, `Patrol`, `Guard`, `Follow`, `Regroup`, `Kite`, `Chase`, `Attack`, `Flee`, and `Kamikaze` semantics, and each SHALL be selectable by at least one rule without Rust changes.

#### Scenario: Previously unreachable behaviors run
- **WHEN** a rule selects `Regroup`, `Follow`, `Flee`, or `Kamikaze`
- **THEN** the behavior executes instead of being unreachable

#### Scenario: Flee moves away
- **WHEN** `Flee` runs
- **THEN** the ship moves away from the target until the configured distance is reached

#### Scenario: Kamikaze detonates
- **WHEN** `Kamikaze` contacts its target
- **THEN** it applies its configured blast and dies

#### Scenario: Attack targets a real entity
- **WHEN** `Attack` runs
- **THEN** it resolves a real target entity and never aims at `Vec2::ZERO`

#### Scenario: Wander uses distinct chances
- **WHEN** `Wander` runs
- **THEN** it uses distinct attack and guard chances and re-rolls on arrival

### Requirement: AI runtime state is saved and invalidated

`AiRuntime { current, target, spawn_position, leader, rule_cooldowns }` SHALL be included in `ShipSnapshot`, restored on load, and invalidated when a target dies or leaves radar.

#### Scenario: Runtime round-trip
- **WHEN** a saved ship has a current behavior and cooldowns
- **THEN** loading restores them

#### Scenario: Target invalidation
- **WHEN** a target dies or leaves radar
- **THEN** the AI re-resolves a valid target

### Requirement: One runtime behavior enum replaces marker components

The system SHALL use one `AiBehavior` enum as the current runtime behavior; the 11 `BehaviorX` marker components, `ShipBehavior`, and `AIShipController` SHALL NOT exist.

#### Scenario: Debug overlay reads runtime behavior
- **WHEN** the debug overlay reads a ship's behavior
- **THEN** it reads `AiRuntime.current` and no marker component

### Requirement: The editor authors AI profiles

The editor SHALL provide an AI panel with a rule list (add, remove, reorder), priority, condition, behavior, and cooldown fields, validation messages, preset buttons (`Aggressive`, `Guard`, `Escort`, `Skirmisher`, `Kamikaze`), a current-behavior preview label, and RON round-trip with the design.

#### Scenario: Rule persists across save and reopen
- **WHEN** a designer adds a rule, saves, and reopens the design
- **THEN** the rule is present with the same values
