## 1. Components and module

- [ ] 1.1 Create `content/ship/ai/` module; add `ai::mod`, `ai::component` with
  one marker component per behavior variant
- [ ] 1.2 Move/adapt `timer` into a `BehaviorTimer` component on the ship
- [ ] 1.3 Update HUD/debug label to read behavior marker components

## 2. Port transitions

- [ ] 2.1 Port `Idle` → `Chase` and `Idle` → `Approach` transitions (template)
- [ ] 2.2 Port `Wander`/`Kiting`, `Chase`, `Regroup`, `Guard`, `Follow`,
  `Patrol`, `Attack`, `Kamikaze`, `Flee`
- [ ] 2.3 Delete `control_ship` and `AIShipBehavior`/`AIShipController`

## 3. Verification and docs

- [ ] 3.1 `cargo check -p core` green
- [ ] 3.2 `docs/ai.md` documents component behaviors
