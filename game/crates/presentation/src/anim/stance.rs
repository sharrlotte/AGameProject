/// Target weights for the upper-body action layer.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct ActionTargets {
    pub aim: f32,
    pub reload: f32,
}

impl ActionTargets {
    /// The action layer strength is the strongest active action.
    pub fn strength(self) -> f32 {
        self.aim.max(self.reload).clamp(0.0, 1.0)
    }
}

/// Derives action targets from the simulation's combat stance and reload state.
///
/// Reload owns the arms while it is active; aim resumes as soon as it ends.
pub fn targets(aiming: bool, reloading: bool) -> ActionTargets {
    if reloading {
        ActionTargets {
            aim: 0.0,
            reload: 1.0,
        }
    } else if aiming {
        ActionTargets {
            aim: 1.0,
            reload: 0.0,
        }
    } else {
        ActionTargets::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stance_targets_prioritize_reload() {
        assert_eq!(
            targets(true, true),
            ActionTargets {
                aim: 0.0,
                reload: 1.0
            }
        );
        assert_eq!(
            targets(true, false),
            ActionTargets {
                aim: 1.0,
                reload: 0.0
            }
        );
        assert_eq!(
            targets(false, true),
            ActionTargets {
                aim: 0.0,
                reload: 1.0
            }
        );
        assert_eq!(targets(false, false), ActionTargets::default());
    }

    #[test]
    fn strength_tracks_the_strongest_action() {
        assert_eq!(targets(true, false).strength(), 1.0);
        assert_eq!(targets(false, true).strength(), 1.0);
        assert_eq!(targets(false, false).strength(), 0.0);
    }
}
