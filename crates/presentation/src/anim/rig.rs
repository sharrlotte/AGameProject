use bevy::prelude::*;

use crate::anim::catalog::IkConfig;

const MAX_CHAIN_DEPTH: usize = 32;

/// One two-bone arm and its optional clavicle.
#[derive(Debug, Clone, Copy, Default)]
pub struct ArmChain {
    pub shoulder: Option<Entity>,
    pub upper_arm: Option<Entity>,
    pub lower_arm: Option<Entity>,
    pub hand: Option<Entity>,
}

impl ArmChain {
    pub fn is_complete(&self) -> bool {
        self.upper_arm.is_some() && self.lower_arm.is_some() && self.hand.is_some()
    }
}

/// Both arms cached at bind time.
#[derive(Debug, Clone, Copy, Default)]
pub struct Arms {
    pub right: ArmChain,
    pub left: ArmChain,
    pub torso: Option<Entity>,
}

/// Links an `AnimationPlayer` to its survivor, visual root, and IK chains.
#[derive(Component, Debug, Clone)]
pub struct PlayerRig {
    pub survivor: Entity,
    pub root: Entity,
    pub arms: Arms,
}

/// Resolves both catalog IK chains and shoulder bones against live named entities.
pub fn find_arms<'a>(ik: &IkConfig, named: impl Iterator<Item = (Entity, &'a str)>) -> Arms {
    let mut arms = Arms::default();
    for (entity, name) in named {
        if name == "Torso" {
            arms.torso = Some(entity);
        } else if name == ik.right_shoulder {
            arms.right.shoulder = Some(entity);
        } else if name == ik.left_shoulder {
            arms.left.shoulder = Some(entity);
        } else if Some(name) == ik.right_chain.first().map(String::as_str) {
            arms.right.upper_arm = Some(entity);
        } else if Some(name) == ik.right_chain.get(1).map(String::as_str) {
            arms.right.lower_arm = Some(entity);
        } else if Some(name) == ik.right_chain.get(2).map(String::as_str) {
            arms.right.hand = Some(entity);
        } else if Some(name) == ik.left_chain.first().map(String::as_str) {
            arms.left.upper_arm = Some(entity);
        } else if Some(name) == ik.left_chain.get(1).map(String::as_str) {
            arms.left.lower_arm = Some(entity);
        } else if Some(name) == ik.left_chain.get(2).map(String::as_str) {
            arms.left.hand = Some(entity);
        }
    }
    arms
}

/// Composes a transform by walking local transforms up to `stop`.
///
/// `GlobalTransform` is stale before transform propagation, so pose systems
/// compose from `Transform` directly. `stop` is excluded from the product,
/// which keeps all results in the visual root's local space. The chain lives
/// on the stack, so this performs no heap allocation.
pub fn compose_world(
    entity: Entity,
    stop: Entity,
    transforms: &Query<&Transform>,
    parents: &Query<&ChildOf>,
) -> Option<Transform> {
    let mut chain = [Transform::IDENTITY; MAX_CHAIN_DEPTH];
    let mut len = 0;
    let mut current = entity;

    for _ in 0..MAX_CHAIN_DEPTH {
        if current == stop {
            break;
        }
        let transform = *transforms.get(current).ok()?;
        chain[len] = transform;
        len += 1;
        current = parents.get(current).ok()?.parent();
    }

    if current != stop {
        return None;
    }

    let mut world = Transform::IDENTITY;
    for index in (0..len).rev() {
        world = world.mul_transform(chain[index]);
    }
    Some(world)
}

pub fn parent_of(entity: Entity, parents: &Query<&ChildOf>) -> Option<Entity> {
    parents.get(entity).ok().map(ChildOf::parent)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn incomplete_arm_chains_are_rejected() {
        let chain = ArmChain::default();
        assert!(!chain.is_complete());

        let complete = ArmChain {
            shoulder: None,
            upper_arm: Some(Entity::PLACEHOLDER),
            lower_arm: Some(Entity::PLACEHOLDER),
            hand: Some(Entity::PLACEHOLDER),
        };
        assert!(complete.is_complete());
    }

    #[test]
    fn find_arms_resolves_both_chains_and_shoulders() {
        let ik = IkConfig::default();
        let mut world = World::new();
        let names = [
            "Shoulder.R",
            "UpperArm.R",
            "LowerArm.R",
            "Palm.R",
            "Shoulder.L",
            "UpperArm.L",
            "LowerArm.L",
            "Palm.L",
            "Head",
        ];
        let entities: Vec<Entity> = names.iter().map(|_| world.spawn_empty().id()).collect();
        let named: Vec<(Entity, &str)> = entities
            .iter()
            .copied()
            .zip(names.iter().copied())
            .collect();

        let arms = find_arms(&ik, named.into_iter());
        assert_eq!(arms.right.shoulder, Some(entities[0]));
        assert_eq!(arms.right.upper_arm, Some(entities[1]));
        assert_eq!(arms.right.lower_arm, Some(entities[2]));
        assert_eq!(arms.right.hand, Some(entities[3]));
        assert_eq!(arms.left.shoulder, Some(entities[4]));
        assert_eq!(arms.left.upper_arm, Some(entities[5]));
        assert!(arms.right.is_complete() && arms.left.is_complete());
    }

    #[test]
    fn compose_world_multiplies_the_parent_chain() {
        let mut world = World::new();
        let root = world.spawn(Transform::IDENTITY).id();
        let parent = world
            .spawn((Transform::from_xyz(1.0, 0.0, 0.0), ChildOf(root)))
            .id();
        let child = world
            .spawn((Transform::from_xyz(0.0, 2.0, 0.0), ChildOf(parent)))
            .id();

        let mut transform_state = world.query::<&Transform>();
        let mut parent_state = world.query::<&ChildOf>();
        let transforms = transform_state.query(&world);
        let parents = parent_state.query(&world);

        let composed =
            compose_world(child, root, &transforms, &parents).expect("composed transform");
        assert!((composed.translation - Vec3::new(1.0, 2.0, 0.0)).length() < 1e-4);
    }

    #[test]
    fn compose_world_rejects_disconnected_entities() {
        let mut world = World::new();
        let root = world.spawn(Transform::IDENTITY).id();
        let orphan = world.spawn(Transform::IDENTITY).id();

        let mut transform_state = world.query::<&Transform>();
        let mut parent_state = world.query::<&ChildOf>();
        let transforms = transform_state.query(&world);
        let parents = parent_state.query(&world);

        assert!(compose_world(orphan, root, &transforms, &parents).is_none());
    }
}
