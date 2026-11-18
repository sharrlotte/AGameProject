## Context

`core` is a binary crate (`core/src/main.rs`, no library target), so `pub` items remain subject to dead-code analysis. Items used only from `#[cfg(test)]` modules are unreachable in a normal build, and the workspace has been silencing the diagnostics with `#[allow(dead_code)]`.

The eight suppressions group into three causes:

| Cause | Sites | Correct response |
|---|---|---|
| Test-only API | `ShipGrid::{entity_at, occupied_cells}`, `ShipDesign::new` | `#[cfg(test)]` or a test-local impl |
| Superseded API | `ShipGrid::refresh_cells`, `save/design::load_design_file` | Delete |
| Spec-ahead / legacy | `ShipMutation` placement variants, `ShipFragment`/`fragments` | Trim to implemented behavior |

`ShipFragment` is already forbidden by `code-foundation`; the mutation placement variants have no producers, and `PlaceBlock` would insert without the overlap check required by `ship-grid`.

## Goals / Non-Goals

**Goals:**
- Remove all eight `#[allow(dead_code)]` sites.
- Leave a production tree with no dead APIs.
- Make the `ShipMutation` contract match implemented behavior.
- Prevent regressions with `dead_code = "deny"`.

**Non-Goals:**
- Implementing runtime placement, repair, or multi-block destruction.
- Renaming `ShipBuildResult`.
- Adding `clippy::allow_attributes` or a source-scanning lint test.
- Changing save schemas, gameplay, assets, or the editor's design-grid flow.

## Decisions

### D1 — Trim `ShipMutation` to `DestroyBlock`
- **Choice**: Remove `DestroyBlocks`, `PlaceBlock`, and `RebuildFromLayout`; update the `ship-grid` requirement to destruction-only and delete the redundant placement-rejection scenario.
- **Alternatives**: keep the variants with `#[expect(dead_code)]`; implement producers; keep only `DestroyBlocks`.
- **Rationale**: `apply_ship_mutations` already groups events per ship, so `DestroyBlocks` adds no batching value. `PlaceBlock` lacks overlap validation and would violate the current spec if used. The overlap guarantee already exists in "Spawn and editor validate ship layouts". Add variants when a producer exists.

### D2 — Delete the superseded functions
- **Choice**: Delete `ShipGrid::refresh_cells` and `save/design::load_design_file`.
- **Alternatives**: keep with `#[expect(dead_code)]`.
- **Rationale**: `refresh_cells` has no caller and the grid is maintained incrementally by `insert_block`/`remove_block`/`rebuild_from_layout`; designs load through `read_designs_dir`/`load_or_seed_designs`. `block-shape-spec` is archived before this change, so its historical references to `refresh_cells` need no edit.

### D3 — Test-only APIs never reach production
- **Choice**: Move `entity_at` and `occupied_cells` into a `#[cfg(test)] impl ShipGrid` inside the grid tests; gate `ShipDesign::new` with `#[cfg(test)]`.
- **Alternatives**: direct field access in tests; a shared `#[cfg(test)] mod test_support`; `#[expect(dead_code)]`.
- **Rationale**: Keeps named helpers readable without exposing production API, and satisfies "dev-only code should not exist in production". `ShipDesign::new` is used by two test modules in the same crate test build, so `cfg(test)` works.

### D4 — Non-optional `ShipBuildResult`, `Option` at the boundary
- **Choice**: Remove `fragments`; make `ship`, `stats`, and `bounds` non-optional; `build_ship_result` returns `Option<ShipBuildResult>`.
- **Alternatives**: keep three `Option` fields; rename to `ShipBuild`.
- **Rationale**: All three values are produced together from a live core; partial states are currently representable but impossible. `Option<Self>` keeps the no-core case explicit. The name stays to avoid documentation churn.

### D5 — Guardrail: `dead_code = "deny"`
- **Choice**: Add `dead_code = "deny"` to `[workspace.lints.rust]`; do not add `clippy::allow_attributes`.
- **Alternatives**: warn only; deny plus `allow_attributes`.
- **Rationale**: Matches the existing deny posture for unused imports and variables. `allow_attributes` would also flag the intentional release-only `cfg_attr(..., allow(unused_variables))` in `ship/system.rs`, unrelated to this change. This change removes all eight `allow(dead_code)` attributes, so the tree is clean.

### D6 — Spec delta shape
- **Choice**: Record the guardrail in proposal, design, and tasks only; the only spec delta is `ship-grid`.
- **Alternatives**: add a `code-foundation` requirement for `dead_code = "deny"`; extend an existing requirement.
- **Rationale**: Specs describe behavior; the lint is build configuration. `code-foundation` already forbids `ShipFragment`.

### D7 — Rename the `ship-grid` requirement
- **Choice**: Replace the four-variant requirement with "Block destruction goes through ShipMutation" and remove the redundant placement-rejection scenario.
- **Alternatives**: keep the old title and narrow the body; keep a generic runtime-change contract.
- **Rationale**: The old title would be false once placement variants are gone. Overlap rejection remains specified by "Spawn and editor validate ship layouts", so no guarantee is lost.

## Risks / Trade-offs

- **[`dead_code = "deny"` may surface additional sites]** → fix them with the same criteria: delete production-unused, `cfg(test)` test-only, implement or expect spec-mandated. Scope is all surfaced dead code, not only the known eight.
- **[`ship-grid` requirement title changes]** → express it as a removed plus added requirement so OpenSpec matches by title.
- **[`ShipDesign::new` disappears from production builds]** → only test code calls it; no runtime path uses it.
- **[`build_ship_result` return type changes]** → four call sites; `cargo test` covers spawn, mutation, and grid paths.
- **[Sequencing]** → implement only after `block-shape-spec` is archived; the current working tree overlaps every file in scope.

## Migration Plan

1. After `block-shape-spec` is archived, apply the deletions and signature changes.
2. Add `dead_code = "deny"`.
3. Run `cargo check` and resolve every surfaced warning using D1–D3 criteria.
4. Run targeted `cargo test` for the ship, block, save, and config modules.
5. Run `cargo dev` in the background, smoke-test block destruction and editor placement, then terminate it.
6. Rollback is a straight revert; no data migration is involved.

## Open Questions

None. All forks were settled with the user: `ShipMutation` trimming, test-helper placement, `ShipDesign::new` gating, `ShipBuildResult` shape and name, deletion of both superseded functions, `dead_code = "deny"`, change scope and timing, core-loss test rewrite, and spec recording.
