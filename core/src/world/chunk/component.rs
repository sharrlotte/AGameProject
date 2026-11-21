use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::constant::STREAM_INTERVAL_SECS;
use crate::save::world::snapshot::{BodySnapshot, DebrisSnapshot, ShipSnapshot};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct ChunkCoord(pub IVec2);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct ChunkData {
    pub enemies: Vec<ShipSnapshot>,
    pub debris: Vec<DebrisSnapshot>,
}

#[derive(Resource, Debug)]
pub struct StreamingTimer(pub Timer);

#[derive(Message, Debug)]
pub struct ChunkStreamError {
    pub coord: ChunkCoord,
    pub detail: String,
}

pub trait StreamSnapshot: Clone {
    fn capture(world: &World, entity: EntityRef) -> Option<Self>;
    fn push_into(self, data: &mut ChunkData);
    fn into_body_snapshot(self) -> BodySnapshot;
}

impl Default for StreamingTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(STREAM_INTERVAL_SECS, TimerMode::Repeating))
    }
}

impl StreamSnapshot for ShipSnapshot {
    fn capture(world: &World, entity: EntityRef) -> Option<Self> {
        ShipSnapshot::capture(world, entity)
    }

    fn push_into(self, data: &mut ChunkData) {
        data.enemies.push(self);
    }

    fn into_body_snapshot(self) -> BodySnapshot {
        BodySnapshot::Ship(self)
    }
}

impl StreamSnapshot for DebrisSnapshot {
    fn capture(world: &World, entity: EntityRef) -> Option<Self> {
        DebrisSnapshot::capture(world, entity)
    }

    fn push_into(self, data: &mut ChunkData) {
        data.debris.push(self);
    }

    fn into_body_snapshot(self) -> BodySnapshot {
        BodySnapshot::Debris(self)
    }
}

impl ChunkCoord {
    pub const fn new(x: i32, y: i32) -> Self {
        Self(IVec2::new(x, y))
    }

    pub fn from_world(pos: Vec2, chunk_size: f32) -> Self {
        Self((pos / chunk_size).floor().as_ivec2())
    }

    pub fn x(&self) -> i32 {
        self.0.x
    }

    pub fn y(&self) -> i32 {
        self.0.y
    }

    pub fn neighborhood_3x3(&self) -> [Self; 9] {
        let x = self.0.x;
        let y = self.0.y;
        [
            Self::new(x - 1, y - 1),
            Self::new(x, y - 1),
            Self::new(x + 1, y - 1),
            Self::new(x - 1, y),
            Self::new(x, y),
            Self::new(x + 1, y),
            Self::new(x - 1, y + 1),
            Self::new(x, y + 1),
            Self::new(x + 1, y + 1),
        ]
    }
}

impl ChunkData {
    pub fn is_empty(&self) -> bool {
        self.enemies.is_empty() && self.debris.is_empty()
    }

    pub fn enemy_count(&self) -> usize {
        self.enemies.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn world_position_maps_to_expected_chunk_coord() {
        let chunk_size = 50_000.0;
        assert_eq!(
            ChunkCoord::from_world(Vec2::new(0.0, 0.0), chunk_size),
            ChunkCoord::new(0, 0)
        );
        assert_eq!(
            ChunkCoord::from_world(Vec2::new(49_999.0, 49_999.0), chunk_size),
            ChunkCoord::new(0, 0)
        );
        assert_eq!(
            ChunkCoord::from_world(Vec2::new(50_000.0, 0.0), chunk_size),
            ChunkCoord::new(1, 0)
        );
        assert_eq!(
            ChunkCoord::from_world(Vec2::new(-1.0, 0.0), chunk_size),
            ChunkCoord::new(-1, 0)
        );
        assert_eq!(
            ChunkCoord::from_world(Vec2::new(-50_000.0, -50_000.0), chunk_size),
            ChunkCoord::new(-1, -1)
        );
        assert_eq!(
            ChunkCoord::from_world(Vec2::new(-50_001.0, 0.0), chunk_size),
            ChunkCoord::new(-2, 0)
        );
    }

    #[test]
    fn neighborhood_3x3_contains_all_surrounding_chunks() {
        let center = ChunkCoord::new(2, -3);
        let neighbors = center.neighborhood_3x3();
        assert_eq!(neighbors.len(), 9);

        let mut expected = Vec::new();
        for y in -4..=-2 {
            for x in 1..=3 {
                expected.push(ChunkCoord::new(x, y));
            }
        }
        for exp in expected {
            assert!(
                neighbors.contains(&exp),
                "neighbors should contain {:?}",
                exp
            );
        }
    }
}
