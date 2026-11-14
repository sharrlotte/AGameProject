## 1. Spec & Docs Sync

- [x] 1.1 Rewrite `openspec/specs/project-layout/spec.md` per the delta spec in this change
- [x] 1.2 Update `docs/07-technical-architecture-save-and-roadmap.md` Section 2 paths to `engine/src/...`

## 2. File Size & Test Placement

- [x] 2.1 Inline tests retained (reverted separate test files; inline #[cfg(test)] kept)
- [x] 2.2 `enemies/tests.rs` left as-is (test file)
- [x] 2.3 Inline tests retained in all production files (no directory-module conversion)
- [x] 2.4 No separate test declarations needed

## 3. Inline Crate Paths

- [x] 3.1 Replace `crate::` paths in `engine/src/presentation/anim/weapon/state.rs` default builder with top-level `use` imports
- [x] 3.2 Audit and fix remaining inline `crate::` paths in production files (`combat/weapon/system.rs`, `config/weapons/roster.rs`)

## 4. Clippy Cleanup

- [x] 4.1 Fix `type_complexity` warnings (extract type aliases) in loader/ai/brain/melee/ranged/ik/mount/camera
- [x] 4.2 Collapse nested `if` statements flagged by clippy in loader/projectile/ik
- [x] 4.3 Fix `manual_contains` in `combat/projectile/state.rs`
- [x] 4.4 Resolve `too_many_arguments` in `enemies/spawn/system.rs::spawn_enemies`
- [x] 4.5 Add `clippy.toml` baseline and verify `cargo clippy -p engine` is clean

## 5. Smoke Test & Verification

- [x] 5.1 Remove `game/tests/app_smoke.rs`
- [x] 5.2 Run `cargo test -p engine` (expect 165+ passing)
- [x] 5.3 Run `cargo clippy -p engine` (expect zero warnings)
- [x] 5.4 Run `cargo build -p game --no-default-features` (expect success)