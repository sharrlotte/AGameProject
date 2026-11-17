## ADDED Requirements

### Requirement: Plugins are declared in mod.rs

Every type that implements `bevy::app::Plugin` SHALL be declared in a file named `mod.rs`. Flat plugin files SHALL be converted to folders containing `mod.rs`, and no plugin SHALL remain declared in a `system.rs` file or a flat module file.

#### Scenario: Plugin relocation
- **WHEN** the source tree is inspected for `impl Plugin` declarations
- **THEN** every declaration is in a `mod.rs` file

#### Scenario: Existing import paths remain stable
- **WHEN** `crate::scene::editor::ShipEditorPlugin` or `crate::content::bullet::BulletPlugin` is imported
- **THEN** the import resolves after the plugin's file is moved into `mod.rs`

### Requirement: Module files have distinct responsibilities

A `system.rs` file SHALL contain functions only and SHALL NOT declare enums, structs, traits, or `impl Plugin` blocks. A `component.rs` file SHALL hold the module's enums, structs, traits, and components. A `lib.rs` file SHALL hold helpers used by sibling modules.

#### Scenario: No types in system.rs
- **WHEN** a `system.rs` file is parsed
- **THEN** it contains no `struct`, `enum`, `trait`, or `impl Plugin` declaration

#### Scenario: New feature follows the roles
- **WHEN** a new feature module is added
- **THEN** its types live in `component.rs`, its systems in `system.rs`, and cross-module helpers in `lib.rs`

### Requirement: Conditional compilation is consolidated

Items gated by the same `#[cfg(...)]` condition within one scope SHALL be grouped into a single conditional block rather than repeated on individual items.

#### Scenario: Dev-gated code
- **WHEN** `main.rs` or `asset/mod.rs` is inspected
- **THEN** items sharing a `dev` or `debug_assertions` gate are inside one block for that scope

#### Scenario: Test-gated items
- **WHEN** a module has several `#[cfg(test)]` items in one scope
- **THEN** they are grouped into one `#[cfg(test)]` block

### Requirement: Crate paths are imported at file scope

Inline fully-qualified `crate::` paths SHALL NOT appear in function signatures, system parameters, or function bodies. Such types and functions SHALL be imported with `use` declarations at the top of the file. Module-level macro invocations are out of scope for this requirement.

#### Scenario: System parameter uses an imported type
- **WHEN** a system takes `AiRuntime` as a query parameter
- **THEN** it uses the imported `AiRuntime` name rather than an inline `crate::content::ship::ai::AiRuntime` path

#### Scenario: Body references an imported constant
- **WHEN** a function builds a transform using `LAYER_SHIP_UI`
- **THEN** it uses the imported constant rather than an inline `crate::constant::LAYER_SHIP_UI` path

### Requirement: Declarations precede executable items

Within each source file, enums, structs, traits, and impls SHALL be declared before functions, and test modules SHALL be at the bottom of the file.

#### Scenario: Component file order
- **WHEN** `content/block/component.rs` is inspected
- **THEN** all type declarations precede the first function and the test module is last

#### Scenario: Scene file order
- **WHEN** `scene/game.rs` or `mission/mod.rs` is inspected
- **THEN** no type is declared after the first function

### Requirement: Asset location has one documented source of truth

`AGENTS.md` SHALL name the actual assets directory used by the project. A settings document SHALL NOT be duplicated outside the assets directory and any stale duplicate SHALL be removed.

#### Scenario: Documented path matches reality
- **WHEN** `AGENTS.md` and the repository root are compared
- **THEN** the documented assets folder is `core\assets` and that folder exists

#### Scenario: No stale settings duplicate
- **WHEN** configuration files are inspected
- **THEN** `core/config/setting.ron` does not exist and only `core/assets/config/setting.ron` defines starter settings

### Requirement: Conventions are guarded against regression

The test suite SHALL fail when a source file violates the plugin-placement, file-role, cfg-consolidation, inline-path, or declaration-order requirements.

#### Scenario: Violation is detected
- **WHEN** a new plugin is declared in a `system.rs` file
- **THEN** the conventions test fails and names the offending file

#### Scenario: Compliant tree passes
- **WHEN** the full test suite runs against the compliant tree
- **THEN** the conventions test passes with no exemptions
