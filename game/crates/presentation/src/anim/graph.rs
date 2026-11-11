use std::collections::HashMap;

use bevy::animation::AnimationTargetId;
use bevy::gltf::Gltf;
use bevy::prelude::*;

use crate::anim::catalog::{AnimCatalog, AnimLayer, ClipDef};

/// Mask group indices are fixed: `0` is lower body, `1` is upper body.
///
/// A node's mask bit means "do not animate this group", so the locomotion
/// layer masks the upper group and the action layer masks the lower group.
pub const LOWER_GROUP: u32 = 0;
pub const UPPER_GROUP: u32 = 1;
pub const LOWER_BIT: u64 = 1 << LOWER_GROUP;
pub const UPPER_BIT: u64 = 1 << UPPER_GROUP;

/// Graph nodes for one catalog clip role.
#[derive(Debug, Clone)]
pub struct ClipNodes {
    pub clip: Handle<AnimationClip>,
    pub lower: Option<AnimationNodeIndex>,
    pub upper: Option<AnimationNodeIndex>,
    pub action: Option<AnimationNodeIndex>,
    pub looped: bool,
    pub speed: f32,
}

impl ClipNodes {
    pub fn lower_node(&self) -> Option<AnimationNodeIndex> {
        self.lower
    }
    pub fn upper_node(&self) -> Option<AnimationNodeIndex> {
        self.upper
    }
    pub fn action_node(&self) -> Option<AnimationNodeIndex> {
        self.action
    }
}

/// All graph nodes addressable by catalog role.
#[derive(Debug, Clone, Default)]
pub struct GraphNodes {
    pub clips: HashMap<String, ClipNodes>,
    pub death: Option<AnimationNodeIndex>,
    pub additive: Option<AnimationNodeIndex>,
}

impl GraphNodes {
    pub fn get(&self, role: &str) -> Option<&ClipNodes> {
        self.clips.get(role)
    }
}

/// Result of one graph build.
pub struct BuildResult {
    pub graph: AnimationGraph,
    pub nodes: GraphNodes,
    pub missing_clips: Vec<String>,
    pub unmatched_mask_bones: Vec<String>,
}

/// Builds the layered graph and registers mask groups from live targets.
///
/// Locomotion clips are duplicated under a lower-only blend and an
/// upper-relaxed blend so the action layer can fade to and from a relaxed pose
/// without freezing the arms.
pub fn build(
    gltf: &Gltf,
    catalog: &AnimCatalog,
    targets: &[(AnimationTargetId, String)],
) -> BuildResult {
    let mut graph = AnimationGraph::new();
    let root = graph.root;

    let lower_blend = graph.add_blend_with_mask(UPPER_BIT, 1.0, root);
    let upper_blend = graph.add_blend_with_mask(LOWER_BIT, 1.0, root);
    let action_blend = graph.add_blend_with_mask(LOWER_BIT, 1.0, root);
    let additive = graph.add_additive_blend_with_mask(LOWER_BIT, 0.0, root);

    let mut matched: Vec<bool> = Vec::new();
    let mut group_bones: Vec<(u32, &str)> = Vec::new();
    if let Some(group) = catalog.mask_group("lower") {
        group_bones.extend(group.bones.iter().map(|bone| (LOWER_GROUP, bone.as_str())));
    }
    if let Some(group) = catalog.mask_group("upper") {
        group_bones.extend(group.bones.iter().map(|bone| (UPPER_GROUP, bone.as_str())));
    }
    matched.resize(group_bones.len(), false);

    for (id, name) in targets {
        for (index, (group, bone)) in group_bones.iter().enumerate() {
            if *bone == name {
                graph.add_target_to_mask_group(*id, *group);
                matched[index] = true;
            }
        }
    }

    let unmatched_mask_bones = group_bones
        .iter()
        .zip(matched.iter())
        .filter(|(_, hit)| !**hit)
        .map(|((_, bone), _)| (*bone).to_string())
        .collect();

    let mut nodes = GraphNodes {
        clips: HashMap::new(),
        death: None,
        additive: Some(additive),
    };
    let mut missing_clips = Vec::new();

    for clip in &catalog.clips {
        let Some(handle) = gltf.named_animations.get(clip.clip.as_str()).cloned() else {
            missing_clips.push(clip.role.clone());
            continue;
        };

        let clip_nodes = match clip.layer {
            AnimLayer::Locomotion => {
                let lower = graph.add_clip(handle.clone(), 0.0, lower_blend);
                let upper = graph.add_clip(handle.clone(), 0.0, upper_blend);
                ClipNodes {
                    clip: handle,
                    lower: Some(lower),
                    upper: Some(upper),
                    action: None,
                    looped: clip.looped,
                    speed: clip.speed,
                }
            }
            AnimLayer::Action => {
                let action = graph.add_clip(handle.clone(), 0.0, action_blend);
                ClipNodes {
                    clip: handle,
                    lower: None,
                    upper: None,
                    action: Some(action),
                    looped: clip.looped,
                    speed: clip.speed,
                }
            }
            AnimLayer::Additive => {
                let action = graph.add_clip(handle.clone(), 0.0, additive);
                ClipNodes {
                    clip: handle,
                    lower: None,
                    upper: None,
                    action: Some(action),
                    looped: clip.looped,
                    speed: clip.speed,
                }
            }
            AnimLayer::Override => {
                let node = graph.add_clip(handle.clone(), 0.0, root);
                if clip.role == "death" {
                    nodes.death = Some(node);
                }
                ClipNodes {
                    clip: handle,
                    lower: None,
                    upper: None,
                    action: Some(node),
                    looped: clip.looped,
                    speed: clip.speed,
                }
            }
        };

        nodes.clips.insert(clip.role.clone(), clip_nodes);
    }

    BuildResult {
        graph,
        nodes,
        missing_clips,
        unmatched_mask_bones,
    }
}

/// Describes a catalog clip without touching the graph; useful for tests.
pub fn clip_role<'a>(catalog: &'a AnimCatalog, role: &str) -> Option<&'a ClipDef> {
    catalog.clip(role)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::anim::catalog::MaskGroupDef;

    fn catalog() -> AnimCatalog {
        AnimCatalog::default()
    }

    #[test]
    fn mask_bits_match_group_indices() {
        assert_eq!(LOWER_BIT, 0b01);
        assert_eq!(UPPER_BIT, 0b10);
    }

    #[test]
    fn mask_groups_are_explicit_bone_lists() {
        let catalog = catalog();
        let lower = catalog.mask_group("lower").expect("lower group");
        let upper = catalog.mask_group("upper").expect("upper group");
        assert!(lower.bones.iter().any(|bone| bone == "Hips"));
        assert!(!lower.bones.iter().any(|bone| bone == "Abdomen"));
        assert!(upper.bones.iter().any(|bone| bone == "Torso"));
        assert!(!upper.bones.iter().any(|bone| bone == "UpperLeg.L"));
    }

    #[test]
    fn clip_roles_are_resolvable_before_graph_build() {
        let catalog = catalog();
        assert_eq!(
            clip_role(&catalog, "idle").map(|clip| clip.clip.as_str()),
            Some("HumanArmature|Man_Idle")
        );
        assert!(clip_role(&catalog, "missing").is_none());
    }

    #[test]
    fn mask_group_definition_keeps_explicit_bones() {
        let group = MaskGroupDef {
            name: "upper".to_string(),
            bones: vec!["Torso".to_string()],
        };
        assert_eq!(group.bones.len(), 1);
    }
}
