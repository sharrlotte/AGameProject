## 1. Extract Shared Objective Advancement Helper and `QuestActivated` Message

- [ ] 1.1 Add `#[derive(Message)] pub struct QuestActivated { pub quest_id: QuestId }` in `core/src/quest/component.rs` and register it in `QuestPlugin`.
- [ ] 1.2 Implement `advance_objectives` in `core/src/quest/lib.rs` (only mutating `QuestTracker` when an objective count changes, sorting active IDs by `QuestId::key()`, returning completed `QuestId`s).
- [ ] 1.3 Update `activate_available` in `core/src/quest/lib.rs` to iterate to a fixed point for transitive `Started` chains and return newly activated `QuestId`s without touching `&mut tracker` when nothing activates.

## 2. Split Monolithic Objective System into Event-Gated Listeners

- [ ] 2.1 Replace `update_quest_objectives` (`core/src/quest/system.rs`) with focused systems: `on_ship_destroyed`, `on_item_collected`, and `check_reach_position` (with `has_active_reach_objectives` run condition).
- [ ] 2.2 Gate `generate_from_discovery` with `.run_if(on_message::<SectorDiscovered>)` and `respawn_player_on_defeat` with `.run_if(on_message::<ShipDestroyed>)`.
- [ ] 2.3 Emit `QuestActivated` from `on_accept_offer` (`core/src/quest/hud.rs`) and `activate_quests`, and re-schedule `activate_quests` reactively on session entry, catalog change, `QuestCompleted`, or `QuestActivated` chained `.after(...)` the objective listeners in `FixedPostUpdate::Death`.
- [ ] 2.4 Throttle `sync_offer_visibility` and add a `has_offered_quests` guard so zero ship scans execute when no offers are pending.

## 3. Verification and Docs

- [ ] 3.1 Migrate and expand unit tests in `core/src/quest/system.rs` verifying each listener independently, idle zero-mutation change-detection preservation, same-tick `QuestCompleted -> activate_quests` chaining, and transitive `Started` activation.
- [ ] 3.2 Update `docs/quests.md` and `docs/architecture.md` (spec only, no notes) documenting the separate event-gated quest listeners and `QuestActivated`.
