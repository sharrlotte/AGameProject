## Context

The repo uses a two-crate layout (`engine` library + `game` binary) with feature folders and 1-word file names. AGENTS.md rules and the project-layout spec are the contract, but drift accumulated: oversized files, co-located tests, inline crate paths, clippy warnings, a stale spec, and a broken smoke test. CI is currently entirely commented out in `.github/workflows/build-games.yml`.

## Goals / Non-Goals

**Goals:**
- Bring every Rust source file under 500 lines.
- Move all `#[cfg(test)]` modules out of production files into sibling `tests.rs`.
- Replace inline `crate::` paths in production code with top-level `use` imports.
- Resolve all 17 clippy warnings and add a `clippy.toml` baseline.
- Sync `openspec/specs/project-layout/spec.md` and `docs/07-technical-architecture-save-and-roadmap.md` to the two-crate reality.
- Remove the Windows-incompatible smoke test.

**Non-Goals:**
- Multiplayer readiness refactor (`Single<Survivor>` stays as documented pragmatic exception).
- Phase 2 features (persistence, automation remain no-op stubs).
- Enabling live CI (workflow stays commented; only clippy config is added).
- Rewriting business logic or changing runtime behavior.

## Decisions

1. **Test relocation strategy.** Move each inline `#[cfg(test)] mod tests` block into a sibling `tests.rs` and declare `#[cfg(test)] mod tests;` in the parent `mod.rs`/file. Rationale: matches existing convention (`combat/tests.rs`, `enemies/tests.rs`); keeps diffs reviewable. Alternative (keep inline) rejected because it conflicts with AGENTS.md.

2. **Oversized file splits.** `presentation/anim/catalog/mod.rs` → move its test module to `catalog/tests.rs`; production part stays. `enemies/tests.rs` (555) → split into `enemies/tests.rs` + `enemies/tests_extra.rs` only if still >500 after relocation; otherwise keep single file. Rationale: minimize churn; prefer one tests file per module.

3. **Inline crate paths.** Add `use` imports at the top of each affected file; keep macro-generated paths inside `assert_ron_config_parses!` macro body (macro hygiene requires `$crate`).

4. **Clippy baseline.** Fix all 17 warnings first, then add `clippy.toml` with `too-many-arguments-threshold` and deny list; no grandfathering. Rationale: clean slate prevents drift; warnings are all mechanical.

5. **Spec sync.** Rewrite `project-layout` spec requirements to describe the two-crate layout and the enforced rules; add a `SUPERSEDED.md`-style note? No — full rewrite preferred since the old multi-crate layout no longer exists.

6. **Smoke test removal.** Delete `game/tests/app_smoke.rs`; the engine crate already covers unit-level init. Manual `cargo run` covers full app. Rationale: cannot run winit on a test thread on Windows without `any_thread`, which is dev-only and not worth the complexity.

## Risks / Trade-offs

- [Risk] Large mechanical refactor risks merge conflicts with in-flight feature work. → Mitigation: do it in one change, applied atomically; coordinate before starting.
- [Risk] Moving tests may break `#[cfg(test)]` visibility of private items. → Mitigation: keep tests in same module tree via `mod tests;` in the same file, preserving `super::*` access.
- [Risk] Clippy fixes may alter control flow (collapsible_if). → Mitigation: behavior-preserving collapses only; run full test suite after.
- [Trade-off] Removing smoke test loses one integration-level signal. → Mitigation: engine tests + manual run; revisit if a headless init harness is added later.

## Migration Plan

1. Sync spec + docs (no code).
2. Split oversized files + relocate tests.
3. Fix inline crate paths.
4. Fix clippy warnings + add clippy.toml.
5. Remove smoke test.
6. Run `cargo test -p engine`, `cargo clippy -p engine`, `cargo build -p game --no-default-features`.

Rollback: revert the single change commit.

## Open Questions

- None outstanding; all decisions captured in explore session.