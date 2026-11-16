## 1. Simulation contract

- [ ] 1.1 Create `simulation.rs` (or `simulation/mod.rs`) with the shared set
  definitions and fixed-schedule ordering contract
- [ ] 1.2 Register the sets in `main.rs` and migrate every plugin to named sets
  in dependency order
- [ ] 1.3 Replace ad-hoc ordering with set-ordering across all plugins
  (content, mission, scene, save, config) and verify `cargo check -p core`

## 2. Marker components

- [ ] 2.1 Delete `EntityType` from `content/entity.rs`
- [ ] 2.2 Update selection to use typed marker components for counts
- [ ] 2.3 Update battle outcome/spawn paths to use typed marker queries
- [ ] 2.4 Sweep any remaining `EntityType` references and remove them

## 3. Typed state split

- [ ] 3.1 Introduce `CameraTarget` (camera follow) and `DeployPayload` (deploy
  ship) resources, delete matching fields from `GameVars`
- [ ] 3.2 Update camera, player control, selection, mission, editor, and AI
  systems to use the typed resources
- [ ] 3.3 Remove empty `GameVars` type if it is no longer shared

## 4. Verification and docs

- [ ] 4.1 `cargo check -p core` passes with zero new warnings
- [ ] 4.2 Update `docs/architecture.md` with named sets and typed resources
- [ ] 4.3 Update `docs/controls.md` or relevant docs if they reference removed
  types
