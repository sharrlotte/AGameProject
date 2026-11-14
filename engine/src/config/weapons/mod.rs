pub mod grip;
pub mod model;
pub mod presentation;
pub mod roster;
pub mod sockets;

pub use grip::{FingerCurl, GripHand, WeaponGripConfig, euler_deg};
pub use model::{ModelSpace, WeaponAnchorConfig};
pub use presentation::{WeaponPose, WeaponRecoil, WeaponReload, WeaponSway, WeaponSwap};
pub use roster::{WeaponDefinition, WeaponRoster};
pub use sockets::{SocketOverride, WeaponSockets};
