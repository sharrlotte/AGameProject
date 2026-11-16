## 1. Grid cache

- [ ] 1.1 Introduce `ship/grid.rs` containing shared `ShipGrid`; move out of
  editor module if needed
- [ ] 1.2 Populate the grid on ship spawn; update on block destroy events
- [ ] 1.3 Replace `cell_map` construction in `blocks_stats` with grid reads

## 2. Verification and docs

- [ ] 2.1 `cargo check -p core` green; fragment behaviour unchanged
- [ ] 2.2 Update `docs/simulation.md` (grid sourced from cache) and add a
  perf note to `docs/rewrite.md`
