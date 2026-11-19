## MODIFIED Requirements

### Requirement: The AI panel edits profiles

The editor AI panel SHALL display `Doctrine` selector buttons (`Auto`, `Brawler`, `Skirmisher`, `Sniper`, `Support`, `Escort`), highlight the active doctrine, update `EditorState.doctrine`, and round-trip `ShipDesign.doctrine` through RON without rule lists or RON preview text.

#### Scenario: Doctrine round-trip
- **WHEN** a designer selects a `Doctrine`, saves the ship, and reopens it
- **THEN** the saved `Doctrine` is loaded and highlighted in the AI panel

### Requirement: Settings are debounced and merged

Settings saves SHALL be debounced and merged with defaults; one settings source of truth (`GameSetting`) SHALL exist and include `ai_level: u8` (`1..=10`). The Settings UI overlay SHALL provide an `AI Skill` `[-]` / `[+]` stepper displaying the current level (`1..=10`) and updating `GameSetting.ai_level`.

#### Scenario: Rapid settings changes
- **WHEN** several settings including `ai_level` change quickly
- **THEN** one debounced write occurs and unspecified values keep defaults

#### Scenario: AI Skill stepper clamps to 1..=10
- **WHEN** the player clicks `-` at level `1` or `+` at level `10` in the Settings overlay
- **THEN** `GameSetting.ai_level` remains clamped within `1..=10` and the displayed value matches
