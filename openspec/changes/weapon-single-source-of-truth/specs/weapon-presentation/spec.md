## MODIFIED Requirements

### Requirement: Data-driven weapon model space and grip
The unified `WeaponDefinition`'s presentation section SHALL declare the model space each weapon was authored
in, and SHALL map the model into weapon space with `model_space`, optional `model_rotation_deg`, and
`model_scale`. The presentation section MAY override socket transforms in model space and MAY declare a grip
with per-hand position, finger direction, palm direction, elbow hint, finger curl, wrist trim, and
weapon-space alignment. Omitted grip fields SHALL be derived from the weapon's sockets, so a new weapon needs
only catalog data and no engine change.

#### Scenario: Converted model maps into weapon space

- **WHEN** a weapon declares `model_space: ForwardNegYUpZ`
- **THEN** the model is rotated so `-Y` model forward becomes `-Z` weapon forward and `+Z` model up becomes
  `+Y` weapon up

#### Scenario: New weapon works without a grip entry

- **WHEN** a weapon omits the grip
- **THEN** the resolver derives a primary grip from the `Grip` socket and a support grip from `Foregrip`, or no
  support grip when the foregrip is missing or too close
