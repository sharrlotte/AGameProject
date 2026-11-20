# booster-shedding Specification

## Purpose
TBD - created by archiving change runtime-state-cleanup. Update Purpose after archive.
## Requirements
### Requirement: Booster shedding has one source of truth
The system SHALL represent booster shedding solely by the presence or absence of the `Disabled` component on booster blocks. `Brain.shed_boosters` and `BrainSave.shed_boosters` SHALL NOT exist. No system SHALL maintain a separate boolean that must agree with those markers.

#### Scenario: Loaded save restores shedding
- **WHEN** a saved ship with shed boosters is restored
- **THEN** shedding is derived from the `Disabled` markers on its boosters, with no separate boolean to reconcile

### Requirement: Shedding is read in one per-tick pass
The AI SHALL build a per-ship shedding set inside the existing per-tick booster pass that also builds `boosters_by_ship`. Decisions SHALL use an O(1) membership lookup. No system SHALL scan a ship's boosters merely to detect or repair shedding state.

#### Scenario: Many ships with boosters
- **WHEN** many ships with boosters are simulated in one tick
- **THEN** shedding detection costs one pass over boosters plus O(1) lookup per ship, not a scan per ship

### Requirement: Markers change only on hysteresis transitions
`Disabled` SHALL be inserted on a ship's boosters when power falls below the shed thresholds and removed when power recovers past the restore thresholds. Neither operation SHALL run on ticks where the shedding state is unchanged.

#### Scenario: Power stays low
- **WHEN** power remains below the shed threshold for several ticks
- **THEN** no additional `Disabled` insert/remove commands are issued after the first transition

