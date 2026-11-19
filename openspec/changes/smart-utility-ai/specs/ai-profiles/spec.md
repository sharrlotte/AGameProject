## MODIFIED Requirements

### Requirement: AI profiles are per-ship and inline

Every `ShipDesign` SHALL contain an optional `doctrine: Doctrine` (`Auto`, `Brawler`, `Skirmisher`, `Sniper`, `Support`, `Escort`, defaulting to `Doctrine::Auto`) instead of inline rule lists (`AiProfile` / `AiRule`). Per-ship behavior SHALL be derived dynamically from live `Capability` introspection, `Skill(1..=10)`, `Doctrine`, and `TeamIntel`.

#### Scenario: Ship design omits doctrine
- **WHEN** a `ShipDesign` RON omits `doctrine`
- **THEN** it deserializes with `Doctrine::Auto` and derives its combat role from its live blocks

#### Scenario: Two identical hulls use different doctrines
- **WHEN** two ships with identical blocks are configured with `Doctrine::Escort` and `Doctrine::Brawler`
- **THEN** each ship executes its respective doctrine without any shared rule list

### Requirement: AI runtime state is saved and invalidated

`Brain { current, spawn_position, think_timer, strafe_sign, strafe_timer, shed_boosters }` along with `Doctrine` and `Skill` SHALL be included in `ShipSnapshot`, restored on load, and invalidated when a target entity no longer exists or is absent from `TeamIntel`.

#### Scenario: Runtime round-trip
- **WHEN** a saved ship has an active `Tactic`, `Skill`, `Doctrine`, `spawn_position`, and `shed_boosters` state
- **THEN** loading restores them accurately

#### Scenario: Target invalidation
- **WHEN** a target ship dies or leaves `TeamIntel`
- **THEN** the AI clears the stale target and re-evaluates via Utility scorers

### Requirement: One runtime behavior enum replaces marker components

The system SHALL use one `Tactic` enum inside `Brain` as the current runtime behavior; `AiBehavior`, `AiRuntime`, `AiProfile`, `AiRule`, `ShipBehavior`, and legacy marker components SHALL NOT exist.

#### Scenario: Debug overlay reads runtime behavior
- **WHEN** the dev debug overlay reads a ship's behavior
- **THEN** it reads `Brain.current`, `Skill`, `Doctrine`, `Capability`, and `Steering`

### Requirement: The editor authors AI profiles

The editor AI panel SHALL provide `Doctrine` selector buttons (`Auto`, `Brawler`, `Skirmisher`, `Sniper`, `Support`, `Escort`) that persist with `ShipDesign` through RON round-trip, replacing the legacy rule list, presets, and RON rule preview.

#### Scenario: Doctrine persists across save and reopen
- **WHEN** a designer selects `Doctrine::Sniper`, saves, and reopens the design
- **THEN** the design loads with `Doctrine::Sniper` selected

## REMOVED Requirements

### Requirement: Rules are prioritized and cooldown-gated
**Reason**: Replaced by block-introspective Utility AI (`Brain` + composable `Scorer`s with hysteresis) in `smart-utility-ai`.
**Migration**: Remove `AiRule`, `AiCondition`, and `rule_cooldowns`; configure global tuning in `core/assets/config/base/ai.ron`.

### Requirement: All authored behaviors are reachable
**Reason**: Replaced by Utility AI `Tactic` variants (`Engage`, `Recharge`, `Retreat`, `Support`, `Regroup`, `Investigate`, `Search`) in `smart-utility-ai`.
**Migration**: Use `Doctrine` overrides and live block composition instead of manual `BehaviorSpec` rules.
