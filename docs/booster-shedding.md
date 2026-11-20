# Booster Shedding Spec

## Single Source of Truth

Booster shedding SHALL be represented only by the presence or absence of `Disabled` on booster blocks. `Brain.shed_boosters` and `BrainSave.shed_boosters` SHALL NOT exist. No system SHALL keep a separate boolean that must agree with those markers.

## Per-Tick Detection

The AI SHALL build a per-ship shedding set inside the existing per-tick booster pass that also builds `boosters_by_ship`. Shedding decisions SHALL use an O(1) membership lookup. No system SHALL scan a ship's boosters merely to detect or repair shedding state.

## Hysteresis

`Disabled` SHALL be inserted when power falls below the shed thresholds and removed when power recovers past the restore thresholds. Insert/remove SHALL occur only on transitions, not on unchanged ticks.

## Ownership

`UiPlugin` SHALL own modal registration. `sync_ship_health` SHALL be registered in `ShipPlugin` in `FixedPreUpdate` / `SimulationSet::Stats` after `StatsUpdateSet`, gated to `SceneState::Game`.
