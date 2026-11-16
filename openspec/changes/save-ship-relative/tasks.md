## 1. Schema

- [ ] 1.1 Define `ShipSave` and `WorldSave` structs in `save/schema.rs`; add
  `WORLD_SCHEMA_VERSION`
- [ ] 1.2 Write RON serializers (save single ship, save world) via `ron`
- [ ] 1.3 Write RON deserializers + `read_version()` strict check

## 2. Load flow

- [ ] 2.1 `spawn_ship_from_save` single entry point; pause-menu load routes
  through it
- [ ] 2.2 World load deletes non-ship actors via existing scene-exit cleanup,
  then spawns from ship files + world.ron

## 3. Save flow

- [ ] 3.1 Pause-menu Save writes per ship + `world.ron` (replacing `test.json`
  entirely)
- [ ] 3.2 Delete the old `ShipSave/BlockSave/...` structs and JSON path
- [ ] 3.3 Docs: `docs/save.md` updated to the RON layouts; `docs/rewrite.md`
  append Slice 6 status

## 4. Verification

- [ ] 4.1 `cargo check -p core` green
- [ ] 4.2 Manual: save during battle → return to Home → Load → verify ship
  state preserved; effects absent
