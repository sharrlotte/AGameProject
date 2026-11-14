## ADDED Requirements

### Requirement: Survivor knockback response

The survivor SHALL accumulate a decaying knockback velocity from the `knockback` field of
`DamageMessage`. Knockback MUST be applied on the horizontal plane, MUST decay exponentially at the
configured rate, and MUST be clamped to a configured maximum. Player movement input MUST remain active
and additive while knockback decays.

#### Scenario: Heavy hit displaces the survivor

- **WHEN** a survivor receives damage with a non-zero knockback impulse
- **THEN** the survivor's horizontal velocity includes a component along the impulse direction

#### Scenario: Knockback decays to zero

- **WHEN** no further knockback is received
- **THEN** the accumulated knockback velocity decays to zero within the configured decay time

#### Scenario: Movement control remains available

- **WHEN** a survivor is under knockback and the player provides movement input
- **THEN** the resulting velocity combines the movement input with the remaining knockback

#### Scenario: Zero knockback has no effect

- **WHEN** a damage message carries a zero knockback impulse
- **THEN** the survivor's knockback velocity is unchanged

#### Scenario: Vertical knockback is ignored

- **WHEN** a damage message carries a knockback impulse with a vertical component
- **THEN** only the horizontal component contributes to survivor displacement
