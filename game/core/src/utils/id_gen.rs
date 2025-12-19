use std::sync::atomic::{AtomicU64, Ordering};

use bevy::ecs::resource::Resource;

#[derive(Resource)]
pub struct AppId {
    pub ship_id: IdGen,
}

impl Default for AppId {
    fn default() -> Self {
        Self {
            ship_id: IdGen::new(),
        }
    }
}

pub struct IdGen {
    id: AtomicU64,
}

impl IdGen {
    fn new() -> Self {
        Self { id: AtomicU64::new(0) }
    }

    pub fn next(&self) -> u64 {
        self.id.fetch_add(1, Ordering::Relaxed)
    }
}
