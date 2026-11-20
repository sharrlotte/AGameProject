## MODIFIED Requirements

### Requirement: Config and mission failures are recoverable
Config and mission loading SHALL return `ConfigState::Error(ConfigError)` with a readable message and error screen instead of panicking. `RonAssetLoader` SHALL execute `Validate::validate` at asset decode time and reject invalid assets before they are registered in the asset storage. Mission failure SHALL return to Home with a message.

#### Scenario: Malformed block config
- **WHEN** `block.ron` is malformed or fails validation on decode
- **THEN** the game shows the config error screen naming the file and validation reason, and does not panic

#### Scenario: Hot reload of invalid config
- **WHEN** a watched `.ron` file is modified on disk with invalid values
- **THEN** `RonAssetLoader` rejects the reload with an error log and the running game retains the last valid config asset
