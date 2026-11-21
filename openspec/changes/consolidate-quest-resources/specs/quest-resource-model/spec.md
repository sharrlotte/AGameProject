## ADDED Requirements

### Requirement: Quest state is split into exactly three lifetime-scoped resources
The quest subsystem SHALL own exactly three runtime resources separated by lifetime and mutability:
1. `QuestConfig` (read-only global authored quest definitions and generation tuning, loaded and validated during `ConfigState::Loading -> Ready`).
2. `QuestTracker` (per-slot persistent runtime records owning both authored progress and generated quest definitions).
3. `QuestOfferUi` (transient HUD projection holding `visible: Vec<QuestId>` and `open: Option<QuestId>`).

`QuestCatalogValue`, `QuestGenConfigValue`, `QuestHandles`, `QuestAssetsReady`, `QuestOfferQueue`, and `VisibleOffers` SHALL NOT exist.

#### Scenario: Reloaded generated quest continues progressing after save and load
- **WHEN** a generated `QuestId::Chunk` quest is active, saved to SQLite, and reloaded into a fresh session where `QuestConfig` contains only authored RON quests
- **THEN** objective updates read `record.def` from `QuestTracker` and advance the reloaded generated quest to completion normally

#### Scenario: Creating a new save slot does not inherit prior slot generated quests
- **WHEN** a generated quest exists in Slot A and the player creates Slot B in the same application session
- **THEN** `QuestConfig.authored` contains no generated entries, `QuestTracker` and `QuestOfferUi` are reset empty, and Slot B receives none of Slot A's generated quests

#### Scenario: QuestOfferUi uses change detection without stale Local caches across scene re-entry
- **WHEN** a manual quest offer is visible, the player exits `SceneState::Game` to Ship Editor and returns to `SceneState::Game`
- **THEN** the offer card UI renders freshly without being suppressed by stale `Local` system state

#### Scenario: Idle frames do not falsely mark QuestTracker or QuestOfferUi changed
- **WHEN** no quest activates, advances, or changes offer visibility during a frame
- **THEN** neither `resource_changed::<QuestTracker>` nor `resource_changed::<QuestOfferUi>` triggers
