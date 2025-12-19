use std::sync::atomic::{AtomicUsize, Ordering};

use bevy::ecs::resource::Resource;

#[derive(Resource, Default)]
pub struct IdGen {
    id: AtomicUsize,
}

impl IdGen {
    fn new() -> Self {
        Self {
            id: AtomicUsize::new(0),
        }
    }

    pub fn next(&self) -> usize {
        self.id.fetch_add(1, Ordering::Relaxed)
    }
}
