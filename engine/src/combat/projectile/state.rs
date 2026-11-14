use bevy::prelude::*;
use crate::common::DamageKind;
use crate::config::weapons::MAX_HITS;

/// Bounded log of entities hit by a projectile, avoiding heap allocations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HitLog {
    entities: [Option<Entity>; MAX_HITS],
    count: usize,
}

impl Default for HitLog {
    fn default() -> Self {
        Self::new()
    }
}

impl HitLog {
    pub const fn new() -> Self {
        Self {
            entities: [None; MAX_HITS],
            count: 0,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    pub fn len(&self) -> usize {
        self.count
    }

    pub fn is_full(&self) -> bool {
        self.count >= MAX_HITS
    }

    pub fn contains(&self, entity: Entity) -> bool {
        self.entities[..self.count].contains(&Some(entity))
    }

    /// Records an entity into the hit log if not already present.
    /// Returns `true` if the log is now full, or `false` if there is remaining capacity.
    pub fn push(&mut self, entity: Entity) -> bool {
        if !self.contains(entity) && self.count < MAX_HITS {
            self.entities[self.count] = Some(entity);
            self.count += 1;
        }
        self.is_full()
    }
}

/// Physical 3D projectile sensor entity fired by weapons.
#[derive(Component, Debug, Clone)]
pub struct Projectile {
    pub owner: Entity,
    pub damage: f32,
    pub initial_damage: f32,
    pub remaining_power: f32,
    pub penalty: f32,
    pub knockback: f32,
    pub direction: Vec3,
    pub radius: f32,
    pub lifetime: f32,
    pub kind: DamageKind,
    pub hit_log: HitLog,
}

#[cfg(test)]
mod tests {
use super::*;

    #[test]
    fn hit_log_insertion_and_contains() {
        let mut log = HitLog::new();
        assert!(log.is_empty());
        assert_eq!(log.len(), 0);
        assert!(!log.is_full());

        let e1 = Entity::from_raw_u32(1).expect("entity");
        let e2 = Entity::from_raw_u32(2).expect("entity");

        let full = log.push(e1);
        assert!(!full);
        assert_eq!(log.len(), 1);
        assert!(log.contains(e1));
        assert!(!log.contains(e2));

        let full = log.push(e2);
        assert!(!full);
        assert_eq!(log.len(), 2);
        assert!(log.contains(e2));
    }

    #[test]
    fn hit_log_duplicate_rejection() {
        let mut log = HitLog::new();
        let e1 = Entity::from_raw_u32(42).expect("entity");

        log.push(e1);
        assert_eq!(log.len(), 1);

        // Pushing duplicate must not increase count
        let full = log.push(e1);
        assert!(!full);
        assert_eq!(log.len(), 1);
    }

    #[test]
    fn hit_log_full_reporting() {
        let mut log = HitLog::new();
        for i in 0..MAX_HITS {
            let entity = Entity::from_raw_u32(i as u32 + 10).expect("entity");
            let is_full = log.push(entity);
            if i + 1 == MAX_HITS {
                assert!(is_full);
                assert!(log.is_full());
                assert_eq!(log.len(), MAX_HITS);
            } else {
                assert!(!is_full);
            }
        }

        // Pushing beyond MAX_HITS does not overflow or panic
        let overflow_entity = Entity::from_raw_u32(999).expect("entity");
        let is_full = log.push(overflow_entity);
        assert!(is_full);
        assert_eq!(log.len(), MAX_HITS);
        assert!(!log.contains(overflow_entity));
    }
}
