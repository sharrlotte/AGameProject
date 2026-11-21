## MODIFIED Requirements

### Requirement: Quests are loaded from validated hot-reloadable RON definitions

The quest system SHALL load `QuestCatalogDef` (`core/assets/config/quest/quests.ron`) and `QuestGenConfig` (`core/assets/config/quest/generation.ron`) inside `ConfigPlugin`'s boot gate (`ConfigState::Loading -> Ready`) with hot-reload support, populating `QuestConfig` (`authored` + `generation`). Validation SHALL reject duplicate or empty quest IDs, empty objective lists, non-positive objective counts or radii, unknown prerequisite IDs, prerequisite cycles, `QuestAcceptance::Manual` on an authored quest, and invalid generation bounds. `QuestConfig.authored` SHALL store authored entries only and SHALL never be mutated by procedural generation; hot reload SHALL update `QuestConfig.authored` and refresh matching `QuestId::Authored` definitions inside `QuestTracker` while preserving objective progress and generated `QuestTracker` records.

#### Scenario: Valid quest catalog loads and validates during boot
- **WHEN** `quests.ron` defines authored quests with unique IDs, valid objectives, and acyclic `requires`
- **THEN** `ConfigPlugin` validates and populates `QuestConfig` before transitioning to `ConfigState::Ready` with no `QuestAssetsReady` polling system

#### Scenario: Cyclic prerequisite graph is rejected
- **WHEN** Quest A requires Quest B and Quest B requires Quest A
- **THEN** boot validation transitions to `ConfigState::Error` naming the cycle

#### Scenario: Manual authored quest is rejected
- **WHEN** an authored quest in `quests.ron` declares `acceptance: Manual`
- **THEN** validation rejects the catalog with an error naming that quest

#### Scenario: Hot reload preserves generated quests and objective progress
- **WHEN** `quests.ron` changes while an authored quest has partial progress and a generated quest exists in `QuestTracker`
- **THEN** `QuestConfig.authored` and matching authored `QuestTracker` definitions update with progress intact, and generated entries in `QuestTracker` remain unchanged
