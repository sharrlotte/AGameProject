pub mod grip;
pub mod model;
pub mod presentation;
pub mod roster;
pub mod sockets;
pub mod stats;

pub use grip::{FingerCurl, GripHand, WeaponGripConfig, euler_deg};
pub use model::{ModelSpace, WeaponAnchorConfig};
pub use presentation::{
    WeaponPresentation, WeaponPresentationPose, WeaponPresentationRecoil, WeaponPresentationReload,
    WeaponPresentationSway, WeaponPresentationSwap,
};
pub use roster::{WeaponDefinition, WeaponRoster};
pub use sockets::{SocketOverride, WeaponSockets};
pub use stats::WeaponStats;
