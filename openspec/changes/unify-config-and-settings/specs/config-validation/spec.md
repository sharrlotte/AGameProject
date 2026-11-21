## MODIFIED Requirements

### Requirement: Config and mission failures are recoverable

Config loading (`game.ron`, `block.ron`, `quests.ron`) SHALL return `ConfigState::Error(ConfigError)` with a readable message naming the failing file and validation reason instead of panicking. `RonAssetLoader` SHALL execute `Validate::validate` at asset decode time and reject invalid assets before they are registered in asset storage.

#### Scenario: Malformed block config
- **WHEN** `block.ron` is malformed or fails validation on decode
- **THEN** the game shows the config error screen naming `config/block.ron` and the validation reason, and does not panic

#### Scenario: Malformed game config section
- **WHEN** any section of `game.ron` (`sim`, `ai`, `minimap`, `skirmish`, `quest_gen`) fails validation on decode
- **THEN** the game shows the config error screen naming `config/game.ron` and the failing section/rule

#### Scenario: Hot reload of invalid config
- **WHEN** a watched `.ron` file (`game.ron`, `block.ron`, or `quests.ron`) is modified on disk with invalid values
- **THEN** `RonAssetLoader` rejects the reload with an error log and the running game retains the last valid `GameConfig` / `BlockRegistry`

### Requirement: Game config values are validated

`game.ron` loading SHALL validate all nested tuning sections (`sim`, `ai`, `minimap`, `skirmish`, `quest_gen`). In `sim`, debris lifetime minimum and maximum SHALL be positive with `max >= min`, `fleet_spacing >= 0`, `auto_save_interval_secs > 0`, and camera zoom bounds ordered and positive. A violation SHALL produce the config error screen naming `config/game.ron` and the failed rule instead of loading the world.

#### Scenario: Inverted debris lifetime range
- **WHEN** `game.ron` sets `sim.debris_lifetime_min_seconds` above `sim.debris_lifetime_max_seconds`
- **THEN** loading reports a config error naming `config/game.ron` and the range rule

#### Scenario: Non-positive debris lifetime
- **WHEN** `game.ron` sets a debris lifetime bound of zero or less
- **THEN** loading reports a config error naming `config/game.ron` and the positivity rule

#### Scenario: Valid game.ron loads all sections
- **WHEN** `game.ron` sets valid `sim`, `ai`, `minimap`, `skirmish`, and `quest_gen` sections
- **THEN** `GameConfig` is populated with all sections before `ConfigState::Ready`
