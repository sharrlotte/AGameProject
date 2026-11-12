use bevy::prelude::*;

use crate::anim::catalog::{IkConfig, TwistConfig};

const MAX_CHAIN_DEPTH: usize = 32;

/// Resolved spine bone entity and its normalized share of upper-body twist.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TwistTarget {
    pub entity: Entity,
    pub weight: f32,
}

/// The bones the support-hand IK reaches for.
#[derive(Debug, Clone, Copy, Default)]
pub struct BoneSet {
    pub upper_arm: Option<Entity>,
    pub lower_arm: Option<Entity>,
    pub hand: Option<Entity>,
    pub target: Option<Entity>,
}

impl BoneSet {
    pub fn is_complete(&self) -> bool {
        self.upper_arm.is_some()
            && self.lower_arm.is_some()
            && self.hand.is_some()
            && self.target.is_some()
    }
}

/// Links an `AnimationPlayer` to its survivor, visual root, IK bones, and spine twist chain.
#[derive(Component, Debug, Clone)]
pub struct PlayerRig {
    pub survivor: Entity,
    pub root: Entity,
    pub bones: BoneSet,
    pub twist_bones: Vec<TwistTarget>,
}

/// Resolves the catalog IK chain and target names against live named entities.
pub fn find_bones<'a>(ik: &IkConfig, named: impl Iterator<Item = (Entity, &'a str)>) -> BoneSet {
    let upper = ik.chain.first().map(String::as_str);
    let lower = ik.chain.get(1).map(String::as_str);
    let hand = ik.chain.get(2).map(String::as_str);

    let mut bones = BoneSet::default();
    for (entity, name) in named {
        if Some(name) == upper {
            bones.upper_arm = Some(entity);
        } else if Some(name) == lower {
            bones.lower_arm = Some(entity);
        } else if Some(name) == hand {
            bones.hand = Some(entity);
        } else if name == ik.target_bone {
            bones.target = Some(entity);
        }
    }
    bones
}

/// Resolves and normalizes catalog spine twist bones against live named entities.
pub fn find_twist_bones<'a>(
    twist: &TwistConfig,
    named: impl Iterator<Item = (Entity, &'a str)>,
) -> Vec<TwistTarget> {
    if !twist.enabled || twist.bones.is_empty() {
        return Vec::new();
    }
    let named_list: Vec<(Entity, &str)> = named.collect();
    let mut resolved = Vec::with_capacity(twist.bones.len());
    let mut total_weight = 0.0;

    for entry in &twist.bones {
        if !entry.weight().is_finite() || entry.weight() <= 0.0 {
            continue;
        }
        if let Some((entity, _)) = named_list.iter().find(|(_, name)| *name == entry.name()) {
            resolved.push(TwistTarget {
                entity: *entity,
                weight: entry.weight(),
            });
            total_weight += entry.weight();
        }
    }

    if total_weight <= 1e-4 {
        return Vec::new();
    }
    for target in &mut resolved {
        target.weight /= total_weight;
    }
    resolved
}

/// Composes a world transform by walking local transforms up to `stop`.
///
/// `GlobalTransform` is stale before transform propagation, so the IK system
/// composes from `Transform` directly. `stop` is excluded from the product,
/// which keeps all results in the visual root's local space.
pub fn compose_world(
    entity: Entity,
    stop: Entity,
    transforms: &Query<&Transform>,
    parents: &Query<&ChildOf>,
) -> Option<Transform> {
    let mut chain: Vec<Transform> = Vec::with_capacity(MAX_CHAIN_DEPTH);
    let mut current = entity;

    for _ in 0..MAX_CHAIN_DEPTH {
        if current == stop {
            break;
        }
        let transform = *transforms.get(current).ok()?;
        chain.push(transform);
        current = parents.get(current).ok()?.parent();
    }

    if current != stop {
        return None;
    }

    let mut world = Transform::IDENTITY;
    for local in chain.iter().rev() {
        world = world.mul_transform(*local);
    }
    Some(world)
}

pub fn parent_of(entity: Entity, parents: &Query<&ChildOf>) -> Option<Entity> {
    parents.get(entity).ok().map(ChildOf::parent)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::anim::catalog::IkConfig;

    #[test]
    fn incomplete_bone_sets_are_rejected() {
        let bones = BoneSet::default();
        assert!(!bones.is_complete());

        let complete = BoneSet {
            upper_arm: Some(Entity::PLACEHOLDER),
            lower_arm: Some(Entity::PLACEHOLDER),
            hand: Some(Entity::PLACEHOLDER),
            target: Some(Entity::PLACEHOLDER),
        };
        assert!(complete.is_complete());
    }

    #[test]
    fn find_bones_resolves_the_catalog_chain_and_target() {
        let ik = IkConfig::default();
        let mut world = World::new();
        let upper = world.spawn_empty().id();
        let lower = world.spawn_empty().id();
        let hand = world.spawn_empty().id();
        let grip = world.spawn_empty().id();
        let other = world.spawn_empty().id();
        let named = [
            (upper, "UpperArm.L"),
            (lower, "LowerArm.L"),
            (hand, "Palm.L"),
            (grip, "Foregrip"),
            (other, "Head"),
        ];
        let bones = find_bones(&ik, named.into_iter());
        assert_eq!(bones.upper_arm, Some(upper));
        assert_eq!(bones.lower_arm, Some(lower));
        assert_eq!(bones.hand, Some(hand));
        assert_eq!(bones.target, Some(grip));
        assert!(bones.is_complete());
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

    #[test]
    fn find_twist_bones_resolves_and_normalizes_weights() {
        let mut twist = TwistConfig::default();
        twist.bones[0].1 = 1.0;
        twist.bones[1].1 = 1.0;
        twist.bones[2].1 = 2.0;

        let mut world = World::new();
        let hips = world.spawn_empty().id();
        let abdomen = world.spawn_empty().id();
        let torso = world.spawn_empty().id();
        let resolved = find_twist_bones(
            &twist,
            [(hips, "Hips"), (abdomen, "Abdomen"), (torso, "Torso")].into_iter(),
        );
        assert_eq!(resolved.len(), 3);
        assert!((resolved[0].weight - 0.25).abs() < 1e-4);
        assert!((resolved[1].weight - 0.25).abs() < 1e-4);
        assert!((resolved[2].weight - 0.50).abs() < 1e-4);
    }
}
