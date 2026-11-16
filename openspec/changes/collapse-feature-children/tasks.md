## 1. Collapse data features

- [ ] 1.1 Texture: attach sprite on the block entity instead of child; drop
  child spawn; verify z-ordering
- [ ] 1.2 Thruster, Storage, EnergyConsumer, EfficiencyProvider: move components
  onto block entity
- [ ] 1.3 Remove `insert_added_block` BFS fix-up; delete `ComponentOwner`;
  sweep references

## 2. Collapse Turret

- [ ] 2.1 Turret: move turret logic/state directly on the block; keep only the
  barrel sprite on a child; barrel has no logic components
- [ ] 2.2 Editor preview (`spawn_placed_block`) aligned to collapsed layout
- [ ] 2.3 Delete child-transform/`Visibility` bookkeeping from collapsed spawn

## 3. Verify and document

- [ ] 3.1 `cargo check -p core` green
- [ ] 3.2 `docs/architecture.md` and `docs/simulation.md` updated
- [ ] 3.3 Manual spawn/destroy smoke (destroy a block with thrusters + turret,
  confirm fragments behave as before)
