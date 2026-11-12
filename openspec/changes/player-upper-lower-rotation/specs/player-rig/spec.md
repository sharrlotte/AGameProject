## MODIFIED Requirements

### Requirement: Animation clip contract
The player glTF SHALL contain clips for idle, forward walk, backward walk, strafe left, strafe right, aim, reload, and death, using names referenced by the animation catalog.

#### Scenario: Catalog clip names resolve
- **WHEN** the animation catalog is validated against the player glTF
- **THEN** every required clip role (`idle`, `walk_fwd`, `walk_back`, `sprint`, `strafe_l`, `strafe_r`, `aim`, `reload`, `death`) resolves to an existing named animation when authored

#### Scenario: Clips are in place
- **WHEN** the locomotion and combat clips are exported
- **THEN** idle, forward walk, backward walk, strafe left, strafe right, aim, reload, and death contain no root translation that moves the character away from the origin
