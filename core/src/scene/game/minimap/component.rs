use bevy::prelude::*;
use uuid::Uuid;

#[derive(Component)]
pub struct MinimapRoot;

#[derive(Component)]
pub struct MinimapRing;

#[derive(Component)]
pub struct MinimapMarkerLayer;

#[derive(Component)]
pub struct MinimapArrow;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct MinimapMarker {
    pub slot: usize,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AnchorCandidate {
    pub entity: Entity,
    pub id: Uuid,
    pub position: Vec2,
    pub heading: f32,
    pub radar_range: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MinimapAnchor {
    Ship(AnchorCandidate),
    Camera(Vec2),
}

impl MinimapAnchor {
    pub fn position(self) -> Vec2 {
        match self {
            Self::Ship(candidate) => candidate.position,
            Self::Camera(position) => position,
        }
    }

    pub fn ship(self) -> Option<AnchorCandidate> {
        match self {
            Self::Ship(candidate) => Some(candidate),
            Self::Camera(_) => None,
        }
    }
}

#[derive(Clone, Copy)]
pub struct MarkerSpec {
    pub offset: Vec2,
    pub color: Color,
}
