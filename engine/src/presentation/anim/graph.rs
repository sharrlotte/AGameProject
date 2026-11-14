use std::collections::HashMap;

use bevy::animation::AnimationTargetId;
use bevy::gltf::Gltf;
use bevy::prelude::*;

use crate::presentation::anim::catalog::{AnimCatalog, AnimLayer, ClipDef};

/// Mask group indices are fixed: `0` is lower body, `1` is upper body.
///
/// A node's mask bit means "do not animate this group", so the upper-body
/// additive layer masks the lower group.
pub const LOWER_GROUP: u32 = 0;
pub const UPPER_GROUP: u32 = 1;
pub const LOWER_BIT: u64 = 1 << LOWER_GROUP;
pub const UPPER_BIT: u64 = 1 << UPPER_GROUP;

/// Graph nodes for one catalog clip role.
#[derive(Debug, Clone)]
pub struct ClipNodes {
    pub clip: Handle<AnimationClip>,
    pub node: Option<AnimationNodeIndex>,
    pub looped: bool,
    pub speed: f32,
}

impl ClipNodes {
    pub fn graph_node(&self) -> Option<AnimationNodeIndex> {
        self.node
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
/// Layers are the full-body locomotion blend, an optional upper-body additive
/// layer, and the death override. Aim and reload are procedural, so they have
/// no clip nodes.
pub fn build(
    gltf: &Gltf,
    catalog: &AnimCatalog,
    targets: &[(AnimationTargetId, String)],
) -> BuildResult {
    let mut graph = AnimationGraph::new();
    let root = graph.root;
    if let Some(root_node) = graph.get_mut(root) {
        root_node.node_type = bevy::animation::graph::AnimationNodeType::Add;
    }

    let base_blend = graph.add_blend(1.0, root);
    let additive = graph.add_additive_blend_with_mask(LOWER_BIT, 1.0, root);

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

        let node = match clip.layer {
            AnimLayer::Locomotion => graph.add_clip(handle.clone(), 1.0, base_blend),
            AnimLayer::Action | AnimLayer::Additive => {
                graph.add_clip(handle.clone(), 1.0, additive)
            }
            AnimLayer::Override => {
                let node = graph.add_clip(handle.clone(), 1.0, base_blend);
                if clip.role == "death" {
                    nodes.death = Some(node);
                }
                node
            }
        };

        nodes.clips.insert(
            clip.role.clone(),
            ClipNodes {
                clip: handle,
                node: Some(node),
                looped: clip.looped,
                speed: clip.speed,
            },
        );
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
    use crate::presentation::anim::catalog::MaskGroupDef;

    fn catalog() -> AnimCatalog {
        AnimCatalog::default()
    }

    fn fake_gltf(catalog: &AnimCatalog) -> Gltf {
        let mut gltf = Gltf {
            scenes: Vec::new(),
            named_scenes: Default::default(),
            meshes: Vec::new(),
            named_meshes: Default::default(),
            materials: Vec::new(),
            named_materials: Default::default(),
            nodes: Vec::new(),
            named_nodes: Default::default(),
            skins: Vec::new(),
            named_skins: Default::default(),
            animations: Vec::new(),
            named_animations: Default::default(),
            default_scene: None,
            source: None,
        };
        for clip in &catalog.clips {
            gltf.named_animations
                .insert(clip.clip.as_str().into(), Handle::default());
        }
        gltf
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

    #[test]
    fn graph_nodes_have_unit_weight() {
        let catalog = catalog();
        let gltf = fake_gltf(&catalog);
        let result = build(&gltf, &catalog, &[]);
        for idx in result.graph.nodes() {
            let node = &result.graph[idx];
            assert!(
                (node.weight - 1.0).abs() < 1e-5,
                "graph node {:?} should have unit weight, got {}",
                idx,
                node.weight
            );
        }
    }

    #[test]
    fn locomotion_clips_share_the_full_body_blend() {
        let catalog = catalog();
        let gltf = fake_gltf(&catalog);
        let result = build(&gltf, &catalog, &[]);

        let idle = result.nodes.get("idle").expect("idle node");
        assert!(idle.graph_node().is_some());
        assert!(result.nodes.death.is_some());
        assert!(result.nodes.additive.is_some());
        assert!(result.nodes.get("aim").is_none());
        assert!(result.nodes.get("reload").is_none());
    }

    #[test]
    fn missing_clips_are_reported_without_nodes() {
        let catalog = catalog();
        let gltf = Gltf {
            scenes: Vec::new(),
            named_scenes: Default::default(),
            meshes: Vec::new(),
            named_meshes: Default::default(),
            materials: Vec::new(),
            named_materials: Default::default(),
            nodes: Vec::new(),
            named_nodes: Default::default(),
            skins: Vec::new(),
            named_skins: Default::default(),
            animations: Vec::new(),
            named_animations: Default::default(),
            default_scene: None,
            source: None,
        };
        let result = build(&gltf, &catalog, &[]);
        assert_eq!(result.missing_clips.len(), catalog.clips.len());
        assert!(result.nodes.clips.is_empty());
        assert!(result.nodes.death.is_none());
    }
}
