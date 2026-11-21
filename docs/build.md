# Dev Build Performance Spec

## Goals

- Keep the edit → check → test → run loop fast for the 186-file `core` crate.
- Keep dependencies optimized so gameplay in `cargo dev` stays representative.
- Keep every dev alias on one feature graph so Cargo artifacts are shared instead of duplicated.

## Non-goals

- Release profile is unchanged: `codegen-units = 1`, `lto = "thin"`, `opt-level = "s"`, `strip = "debuginfo"`.
- Bevy default features are unchanged; the unused `3d` feature set stays enabled. Removing it is a cold-build-only win and was not adopted.
- The `core` crate is not split into workspace members.
- No production behavior changes; everything here is dev-only configuration.

## Decisions (approved)

- `profile.dev.opt-level = 0`: the crate's own code compiles without optimization during development. Game logic is not the frame-time bottleneck; the optimized Bevy dependencies are.
- `profile.dev.debug = 0`: dev artifacts carry no debug information. Accepted trade-off: backtraces have no symbols.
- `profile.dev.package."*".opt-level = 3`: dependencies stay optimized and cached. Their cost is paid once, not per edit.
- The `dev` cargo feature enables `bevy/dynamic_linking`. Bevy 0.20.1 defines `dynamic_linking = ["dep:bevy_dylib", "bevy_internal/dynamic_linking"]` and documents the feature path as the recommended way to enable dynamic linking; the manual `bevy_dylib` dependency plus `use bevy_dylib;` is explicitly unrecommended. Because it lives inside `dev`, release builds are unaffected.
- `bevy_remote`, `file_watcher`, `bevy_brp_extras`, and `bevy-inspector-egui` stay in the `dev` feature with dynamic linking. Splitting them into a separate feature would let `dc`/`dct` skip the `#[cfg(feature = "dev")]` code that `cargo dev` actually runs, so the check loop could miss real compile errors.
- Verification workflow: `cargo dc` (`check --features dev`) and `cargo dct` (`check --tests --features dev`) after each integration; `cargo t` (`test --features dev`) for real tests; `cargo dev` only for visual or gameplay verification. `AGENTS.md` records this rule.

## Configuration

```toml
# Cargo.toml
[profile.dev]
opt-level = 0
debug = 0

[profile.dev.package."*"]
opt-level = 3
```

```toml
# core/Cargo.toml
[features]
dev = [
    "bevy/bevy_remote",
    "bevy/file_watcher",
    "bevy/dynamic_linking",
    "dep:bevy_brp_extras",
    "dep:bevy-inspector-egui",
]
```

```toml
# .cargo/config.toml
[alias]
dev = "run --features dev"
db = "build --features dev"
dc = "check --features dev"
dct = "check --tests --features dev"
t = "test --features dev"
```

## Expected effect

```
edit → cargo dc    : metadata only, no codegen, no link      ← fast path
     → cargo dct   : same, including test targets             ← fast path
     → cargo t     : real tests, same feature graph, no thrash
     → cargo dev   : own code at opt-level 0, no debug info,
                     Bevy in a DLL → cheaper codegen and relink
```

## Verification

- `cargo dc` and `cargo dct` succeed after a source edit without a long codegen or link phase.
- `cargo dev` launches with Bevy loaded from `bevy_dylib`.
- `cargo t` uses the same feature graph as `dev`; switching between aliases does not trigger a full rebuild.
- One full rebuild is expected when the profile change is first applied.

## Risks

- Windows MSVC dynamic linking together with `bevy_rapier2d`, `bevy_hanabi`, `bevy_brp_extras`, and `bevy-inspector-egui` must be confirmed by an actual `cargo dev` launch; `cargo check` does not link.
- `debug = 0` removes symbol information from backtraces.
- Changing `debug` invalidates existing dev artifacts; the first build after this change is expensive.
