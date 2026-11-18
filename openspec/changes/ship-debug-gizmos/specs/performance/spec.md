## MODIFIED Requirements

### Requirement: Debug UI and ammo icons update on change

Debug ship text SHALL be compiled only with the `dev` feature, SHALL be rendered only for ships in `ShipDebugSelection`, and SHALL update on change; ammo warning icons SHALL be children of blocks or repositioned on change, not every tick.

#### Scenario: Debug overlay disabled
- **WHEN** the `dev` feature is not compiled or no ship is selected
- **THEN** no debug text is rebuilt per tick
