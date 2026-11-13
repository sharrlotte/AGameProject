## 1. Unified Weapon Definition

- [x] 1.1 Add unified `WeaponDefinition` type in `engine::config` with nested `stats` and `presentation` and top-level cross-cutting fields
- [x] 1.2 Add `RonConfig` implementation and register `game/assets/config/weapon/weapon.ron`
- [x] 1.3 Rewire combat systems to consume stats from the unified definition

## 2. Inventory and Presentation Rewiring

- [x] 2.1 Change `WeaponInventory` slots to hold the unified definition plus ammo
- [x] 2.2 Rewire presentation mount, pose, kick, sway, reload, and swap systems to consume the unified definition presentation section
- [x] 2.3 Delete split weapon types and fallback defaults

## 3. Runtime Wiring

- [ ] 3.1 Wire dev stage and AnimTest to the prod-loaded roster (first entry)
- [ ] 3.2 Decouple presentation from simulation components via events and presentation-scoped state

## 4. Docs and Specs

- [ ] 4.1 Update docs and specs
