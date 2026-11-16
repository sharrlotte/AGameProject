## Context

Each block today spawns N+1 entities: the block, then one child per feature.
Child entities exist because `Feature::spawn` needed its own `Transform` /
`Visibility` for sprite-layer rendering, and because radar/turret/`Radar.targets`
selection hit blocks via their transform position (children had own
`GlobalTransform` at `Transform::default()` relative to parent, made to work via
`insert_added_block`). `BlockOwner`/`ComponentOwner` then rebuilt the two-way
edge, at the cost of a per-spawn BFS pass and a `ComponentOwner` component that
exists only as a back-pointer, never queried as a value.

The children do not need to exist except for the turret barrel (an independent
rotating Transform). This change removes them everywhere else.

## Goals / Non-Goals

**Goals:**
- One block entity per block; all feature components (`Health`, `Team`,
  `BlockOwner`, `Sprite`, `Block`/`BlockState`, `Thruster`, `Storage`,
  `EnergyConsumer`, `EfficiencyProvider`, `PowerUsage*`, `Radar`, `Turret`,
  `TurretState`) live on it.
- Only the rotating turret barrel sprite keeps a child entity (it has its own
  `Transform` rotating around the block).
- Delete `ComponentOwner`, delete `insert_added_block`, delete the child-BFS
  ownership fix-up.
- `blocks_stats` remains the connectivity oracle (re-scanning from scratch per
  event, which is the current behaviour); performance rework is Slice 5.
- Editor spawn path aligns: editor spawns blocks with the same collapsed
  components (no parallel child semantics to drift again).

**Non-Goals:**
- Stats/performance caching (Slice 5).
- World-level save redesign (Slice 6).
- Collider/transform changes: blocks stay exactly where they were.

## Decisions

- **Turret barrel sprite stays a child.** Reason: it rotates relative to the
  block; Bevy applies parent rotation to children automatically, and that's a
  free transform stack. Replacing with an inline custom rotation model is extra
  user-facing complexity for zero gain. Collapsed Texture/Thruster/Storage etc.
  stay on the block.
- **Radar and TurretState stay on the block, not on the barrel child.** The
  barrel child has no logic; keeps intersection tests cheap and query filters
  simple.
- **No transient ownership components in the collapsed path.** Consumers that
  needed `ComponentOwner` were doing a two-hop navigation that the flattened
  layout makes impossible to need; they query directly instead.
- **`Texture` keeps its child as a `Sprite` on the block.** Since the block
  entity already carries `Transform` + `GlobalTransform` + `Visibility`, the
  sprite is attached to the same render node — one draw call shape instead of
  two.
- **Break apart `Feature` `insert` role-based dispatch** into a plain helper
  accepting the block entity. No `enum Feature` dispatch remains at runtime —
  the boundary lives solely in the `.ron` → `into_def` conversion.

## Risks / Trade-offs

- [Risk] Parent sprite + child barrel rotation visual ordering changes. →
  Mitigation: explicit z-layers preserved (`LAYER_BLOCK` < barrel overlay
  `LAYER_TURRET_OVERLAY`); verify with one visual pass.
- [Risk] Queries that expected `With<ComponentOwner>` to exclude non-block
  children silently mis-query. → Mitigation: lint every `ComponentOwner` site
  in one mechanical sweep with search; no runtime behaviour diff expected.
- [Risk] Editor spawn path divergence if rewritten separately. → Mitigation:
  route editor spawns through the same `Feature::into_components` helper.

## Migration Plan

1. Collapse in dependency order: Texture → Thruster → Storage → Energy →
   EfficiencyProvider; then Turret (delete `ComponentOwner` + BFS fix-up last).
2. Remove `ComponentOwner` entirely, sweep remaining references.
3. `cargo check -p core`; verify editor spawn; verify spawn/destroy loop
   behaviour (fragmentation unchanged).

Rollback: revert this slice — no format changes.
