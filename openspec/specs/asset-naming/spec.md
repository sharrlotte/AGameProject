# Asset Naming

## Purpose

Shipped assets follow one filename convention, the asset tree contains only referenced assets, and development builds warn about naming violations without failing startup.

## Requirements

### Requirement: Asset filenames use lowercase kebab-case

Every shipped asset file under `core/assets` SHALL use a lowercase kebab-case stem and a lowercase extension. Mixed case, PascalCase, UUID stems, and uppercase extensions SHALL NOT be used for shipped assets.

#### Scenario: Valid asset name
- **WHEN** an asset file named `texture/block/hull.png` is inspected
- **THEN** it conforms to the naming requirement

#### Scenario: Uppercase extension is removed
- **WHEN** the asset tree is inspected after this change
- **THEN** no file named `texture/block/steel.PNG` exists and `texture/block/steel.png` exists

### Requirement: Unreferenced assets are removed

The active asset tree SHALL contain only assets referenced by the game's configuration or code. The procedural background SHALL NOT depend on image files, and the workspace-root `assets` directory SHALL NOT contain a `config/setting.ron` copy.

#### Scenario: Orphan images are gone
- **WHEN** `core/assets/texture/background/` and `core/assets/texture/block/` are inspected after this change
- **THEN** none of `027078b0-a55c-4139-bd9c-c17709b26df7.png`, `background1.png`, `background2.png`, `Nebula1.png`, `Nebula2.png`, `Nebula3.png`, `Stars.png`, `block-scrap.png`, or `ripple.png` are present

#### Scenario: Single settings file
- **WHEN** the workspace asset roots are inspected after this change
- **THEN** `core/assets/config/setting.ron` exists and no root-level `assets/config/setting.ron` exists

### Requirement: Asset names are validated in development builds

A dev-only validator SHALL warn about every asset filename that violates the lowercase kebab-case rule. The validator SHALL NOT be registered in production builds and SHALL NOT fail startup.

#### Scenario: Misnamed file in a dev build
- **WHEN** a dev build starts with an asset file named `texture/block/Nebula1.png`
- **THEN** the validator emits a warning naming that file and the game continues to start

#### Scenario: Production build has no validator
- **WHEN** a production build is inspected for `AssetCheckPlugin`
- **THEN** no asset-name validation plugin is registered

#### Scenario: Valid tree produces no warnings
- **WHEN** a dev build starts with the asset tree produced by this change
- **THEN** the validator emits no filename warnings
