## Why

AGENTS.md defines hard rules (files ≤500 lines, tests in separate files, no inline `crate::` paths, SOLID, no unwrap in prod) and the project-layout spec mandates a crate structure, but the codebase has drifted: two files exceed 500 lines, 33 files carry inline tests, inline fully-qualified crate paths remain in production builders, clippy reports 17 warnings with no gating config, the project-layout spec still describes the pre-collapse multi-crate layout, and the smoke test fails on Windows. The spec-as-source-of-truth workflow is undermined. This change brings code, docs, and specs back into alignment in one sweep.

## What Changes

- **Modified** `project-layout` capability: rewrite spec requirements to reflect the actual two-crate (`engine`/`game`) layout, current module tree, and the established file-size/test-placement rules.
- Split the two oversized files (`presentation/anim/catalog/mod.rs`, `enemies/tests.rs`) and relocate inline `#[cfg(test)]` modules in all 33 affected files into sibling `tests.rs`.
- Remove inline `crate::` fully-qualified paths from production code (notably `presentation/anim/weapon/state.rs` default builder) in favor of top-level `use` imports.
- Fix all 17 current clippy warnings (type_complexity, collapsible_if, manual_contains, too_many_arguments) and add a `clippy.toml` baseline so new warnings fail.
- Remove `game/tests/app_smoke.rs` (winit main-thread incompatibility) and rely on engine unit tests + manual runs.
- Update `docs/07-technical-architecture-save-and-roadmap.md` paths to match the two-crate tree.
- Keep `Single<…, With<Survivor>>` usage in simulation as a documented pragmatic exception (multiplayer prep deferred).
- Keep `PersistencePlugin` and `AutomationPlugin` as no-op stubs (Phase 2 scaffolding).

## Capabilities

### New Capabilities
- (none — this is a refactor/alignment change, no new runtime capability)

### Modified Capabilities
- `project-layout`: requirements updated to the two-crate layout, current module tree, and enforced file-size/test-placement/lint rules.

## Impact

- Affected code: ~33 files with inline tests, `presentation/anim/catalog/mod.rs`, `enemies/tests.rs`, `presentation/anim/weapon/state.rs`, `combat/weapon/system.rs`, `config/weapons/roster.rs`, plus clippy-touched files.
- Affected specs: `openspec/specs/project-layout/spec.md`.
- Affected docs: `docs/07-technical-architecture-save-and-roadmap.md`.
- Affected tests: removal of `game/tests/app_smoke.rs`; relocation of inline tests.
- Dependencies: none new; clippy config only.