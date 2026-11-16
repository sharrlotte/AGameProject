use bevy::prelude::*;

/// Named simulation stage slots, shared by the fixed and frame pipelines.
///
/// Pipelines (in Bevy's built-in order):
/// - Fixed: `FixedFirst` → `FixedPreUpdate` → `FixedUpdate`
///   → `FixedPostUpdate` → `FixedLast`
/// - Frame: `PreUpdate` → `Update` → `PostUpdate` → `Last`
///
/// Every ordering-sensitive system belongs to exactly one set:
/// **Input → Simulate → Apply → Cleanup**.
///
/// - `Input`: poll devices / read window / UI interaction / apply settings
/// - `Simulate`: think, integrate, collide, write derived state
///   (stats, AI, turrets, radar, lifetimes, camera follow)
/// - `Apply`: turn events and damage into lasting state (damage, drops,
///   block fragmentation, spawn/despawn bookkeeping)
/// - `Cleanup`: despawn dead entities, scene teardown, per-frame hygiene
///
/// Sets exist so that ordering is documented and reviewable in one place,
/// not assembled inside plugin implementations.
#[derive(SystemSet, Debug, Clone, Eq, PartialEq, Hash)]
pub enum SimulationSet {
    Input,
    Simulate,
    Apply,
    Cleanup,
}
