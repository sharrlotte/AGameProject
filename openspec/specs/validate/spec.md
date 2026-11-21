# validate Specification

## Requirements

### Requirement: Config structs store validated wrapper types
Config structs SHALL store validated wrapper types directly rather than raw `f32`/`u32` primitives, with readers accessing values via `.get()`.

#### Scenario: Invalid config rejected at parse time
- **WHEN** a RON config file contains an out-of-bounds scalar, inverted range, or invalid cross-field combination
- **THEN** deserialization fails immediately without constructing the config struct
