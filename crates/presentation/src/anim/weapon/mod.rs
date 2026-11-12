pub mod catalog;
pub mod mount;
pub mod pose;
pub mod state;

pub use catalog::{
    RecoilConfig, ReloadConfig, SocketOverride, SwapConfig, SwayConfig, WeaponAnchorConfig,
    WeaponCatalog, WeaponPose, WeaponProfile, WeaponSockets,
};
pub use mount::{init_weapon_anchors, resolve_weapon_sockets, sync_weapon_mounts};
pub use pose::{pose_player_weapon, update_weapon_kick, update_weapon_swap};
pub use state::{
    PlayerWeaponAnchor, ResolvedSockets, WeaponMount, WeaponPoseState, add_kick, compose_weapon,
    decay_kick, reload_offset, swap_curve, sway_offset, weapon_pose,
};
