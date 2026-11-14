pub mod grip;
pub mod model;
pub mod pierce;
pub mod presentation;
pub mod roster;
pub mod sockets;

pub use grip::{FingerCurl, GripHand, WeaponGripConfig, euler_deg};
pub use model::{ModelSpace, WeaponAnchorConfig};
pub use pierce::{MAX_HITS, MIN_PIERCE_COST, PierceConfig};
pub use presentation::{WeaponPose, WeaponRecoil, WeaponReload, WeaponSway, WeaponSwap};
pub use roster::{WeaponDefinition, WeaponRoster};
pub use sockets::{SocketOverride, WeaponSockets};
