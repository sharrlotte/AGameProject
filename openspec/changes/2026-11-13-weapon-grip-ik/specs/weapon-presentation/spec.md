## ADDED Requirements

### Requirement: Data-driven weapon model space and grip
Every weapon profile SHALL declare the model space it was authored in, and SHALL map the model into
weapon space with `model_space`, optional `model_rotation_deg`, and `model_scale`. A profile MAY override
socket transforms in model space. A profile MAY declare a grip with per-hand position, finger direction,
palm direction, elbow hint, finger curl, wrist trim, and weapon-space alignment. Omitted grip fields SHALL
be derived from the weapon's sockets, so a new weapon needs only catalog data and no engine change.

#### Scenario: Converted model maps into weapon space

- **WHEN** a profile declares `model_space: ForwardNegYUpZ`
- **THEN** the model is rotated so `-Y` model forward becomes `-Z` weapon forward and `+Z` model up becomes `+Y` weapon up

#### Scenario: Socket override replaces the baked socket

- **WHEN** a profile lists a socket override
- **THEN** the resolved socket node uses the authored model-space transform instead of the one baked into the GLB

#### Scenario: New weapon works without a grip entry

- **WHEN** a profile omits `grip`
- **THEN** the resolver derives a primary grip from the `Grip` socket and a support grip from `Foregrip`, or no support grip when the foregrip is missing or too close

#### Scenario: Support grip can be disabled

- **WHEN** a profile sets `support: None`
- **THEN** the weapon is treated as one-handed and the left arm is not solved to it

#### Scenario: Per-hand grip is independent

- **WHEN** a profile authors different primary and support finger, palm, elbow, wrist, or align values
- **THEN** each hand resolves and solves with its own values

#### Scenario: Invalid grip data is rejected

- **WHEN** a grip contains a non-finite value, a zero-length direction, or a finger curl beyond 120 degrees
- **THEN** catalog validation fails and the runtime keeps the previous valid configuration
