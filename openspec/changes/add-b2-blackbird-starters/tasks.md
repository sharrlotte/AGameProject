## 1. Starter design data

- [x] 1.1 Author `core/assets/config/starter/b2.ron`: v2 design named `B2` with a fresh UUID, broad flying-wing silhouette from existing hull and triangle blocks, one 3×3 `core`, two `colaris` turrets, more hull than thrusters, reactors and batteries sized to demand, and the stand-off AI profile (`Attack` initial; `Flee` at HP < 0.2 with distance 800 and cooldown 8 s; `Kite(4500)` within 6000).
- [x] 1.2 Author `core/assets/config/starter/blackbird.ron`: v2 design named `Blackbird` with a fresh UUID, long slender dart silhouette from existing hull and triangle blocks, one 3×3 `core`, exactly one `laser_turret`, multiple `thruster_small` blocks and one `booster`, a lighter power plant than `B2`, and the skirmish AI profile (`Chase` initial; `Flee` at HP < 0.4 with distance 1200 and cooldown 5 s; `Kite(2500)` within 4000).

## 2. Seeding and mission registration

- [x] 2.1 Add `B2` and `BLACKBIRD` `include_str!` constants to `core/src/save/design/lib.rs` and extend both seeding loops so the two designs are written on a fresh install.
- [x] 2.2 Extend the `starters_are_valid_documents` test in `core/src/save/design/lib.rs` to cover both new designs.
- [x] 2.3 Update `core/assets/config/mission/quick-battle.ron` so the player remains `Interceptor` and the two enemies are `B2` and `Blackbird`.

## 3. Validation and verification

- [ ] 3.1 Confirm both layouts pass `validate_ship` (one core, orthogonally connected, no overlaps) and power feasibility; adjust cell coordinates and block counts within the agreed silhouettes and loadouts until valid.
- [ ] 3.2 Run the starter and save tests, then a `cargo dev` session: confirm both ships spawn, their silhouettes read as a flying wing and a dart, their AI follows the authored profiles, and Quick Battle resolves the `Interceptor` vs `B2` + `Blackbird` roster.

## 4. Documentation

- [ ] 4.1 Update the starter designs section of `docs/simulation.md` with the `B2` and `Blackbird` composition and role descriptions, spec-style only.
- [ ] 4.2 Update the used-assets list in `docs/assets.md` with the two new starter config paths, spec-style only.
