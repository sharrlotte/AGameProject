## Context

The crate currently compiles after the Bevy 0.20 port, but the fixed-update and
frame update pipelines are wired implicitly: plugins register systems into
`FixedFirst`/`FixedPreUpdate`/`FixedUpdate`/`FixedPostUpdate`/`FixedLast` and
`Update`/`PostUpdate` without a declared contract. A handful of cross-cutting
ordering budgets (efficiency → stats → physics → apply → render) live inside
their plugins with no single place to read them, and invisible ordering bugs
have already bitten once (efficiency buffs applied after the base reset).

`EntityType` is a single enum used as (a) a spawn/lifetime tag, (b) the
selection map key in the game scene, (c) an "is ship" check in battle outcome
detection. One enum, three unrelated meanings — awkward to extend, hard to
replace with dedicated marker types, and imposes match statements in unrelated
files.

`GameVars` is a grab-bag resource holding the deploy payload (`current_ship`),
cursor/camera-target state (`controlling`, `following`), used by the camera,
player control, selection, mission spawn, and AI systems.

## Goals / Non-Goals

**Goals:**
- Name and document the simulation pipeline (`FixedFirst` → … → `PostUpdate`),
  with every ordering-sensitive system participating via a set.
- Eliminate enum-tag components where a plain marker or query filter fits;
  selection and outcome checks must not depend on a shared enum.
- Split grab-bag state into typed resources so AI/UI/camera stop depending on
  unrelated fields.
- Keep the runtime behaviour identical (this is a hygiene/refactor slice).

**Non-Goals:**
- Gameplay, balance, or AI-behaviour changes.
- Any migration of blocks/items to `.ron` or registry-symbol ids (Slice 2).
- Component/feature restructure (Slice 3).
- Save-format work (Slice 6).

## Decisions

- **Ordering contract as one `SimulationSet` enum** in a single module
  (`simulation.rs`), with `configure_sets(FixedUpdate, (A, B, C).chain())`-style
  anchors in the plugin entry. Rationale: gives future slices a single file
  to review ordering, rather than a per-plugin hunt.
  - Alternative considered: per-plugin ordering only. Rejected — same tribal
    knowledge problem, redistributed.
- **Marker components over shared tag enums.** `EntityType` is removed; spawn
  sites already know what they spawn, and queries use the existing marker
  components (`Ship`, `Block`, etc.) or a tiny purpose-built tag where the
  distinction is genuinely missing.
  - Alternative considered: keep `EntityType`, just shrink it. Rejected — same
    accidental-coupling problem recurs in every new system that touches it.
- **Split `GameVars` into typed resources** (e.g., `CameraTarget`,
  `DeployPayload`) instead of one struct. Rationale: camera and battle systems
  currently depend on each other's fields through a single `Res<GameVars>`,
  which needlessly widens rebuild scopes and confuses testing.
  - Alternative considered: leave `GameVars` alone and document it. Rejected —
    documentation cannot fix compile-time coupling.
- **`SceneEntity`/`SceneState` stay as-is.** They already serve their purpose
  and are not on the critical path for Slice 2/6.

## Risks / Trade-offs

- [Risk] Moving systems into named sets changes interaction order in ways that
  only manifest as hitches or rare logic bugs. → Mitigation: enumerate every
  system migration in the spec and tasks; `cargo check -p core` must pass;
  runtime smoke pass by user before merge.
- [Risk] Deleting `EntityType` touches saves? No — `EntityType` is never
  serialized. No format dependency.
- [Risk] Named sets introduce a global ordering convention that fights Bevy's
  ability to run things in parallel; over-chaining could cost frame time.
  → Mitigation: chain only the true dependencies; leave independent sets free.

## Migration Plan

1. Add `SimulationSet` module and wire plugins to named sets in `main.rs`.
2. split `GameVars` into typed resources; update all readers/writers.
3. Replace `EntityType` usage; delete enum; update docs.
4. `cargo check -p core` green; `docs/architecture.md` updated in same commit.

Rollback: revert the commit — no schema/format changes to undo.
