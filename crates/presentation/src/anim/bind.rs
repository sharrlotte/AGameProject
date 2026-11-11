use bevy::animation::AnimationTargetId;
use bevy::gltf::Gltf;
use bevy::prelude::*;
use bevy::world_serialization::WorldAssetRoot;
use data::ConfigHotReloadedMessage;
use world::Survivor;

use crate::config::{PlayerModelConfig, VisualsConfig};

use super::{
    PlayerAnimAssets, PlayerModelChild, PlayerModelSpawned, PlayerVisualRoot,
    catalog::AnimCatalog, director, graph, rig, rig::PlayerRig,
};

pub fn start_loading_player_model(
    asset_server: Res<AssetServer>,
    cfg: Res<VisualsConfig>,
    mut assets: ResMut<PlayerAnimAssets>,
) {
    load_player_gltf(&asset_server, &cfg.player.model_path, &mut assets);
}

fn load_player_gltf(asset_server: &AssetServer, model_path: &str, assets: &mut PlayerAnimAssets) {
    assets.loaded_path = model_path.to_string();
    assets.gltf = Some(asset_server.load(model_path.to_string()));
    assets.clear_built();
}

fn model_child_transform(cfg: &PlayerModelConfig) -> Transform {
    Transform::from_xyz(0.0, cfg.y_offset, 0.0)
        .with_scale(Vec3::splat(cfg.scale))
        .with_rotation(Quat::from_rotation_y(cfg.yaw_offset_deg.to_radians()))
}

pub fn sync_player_visuals_hot_reload(
    mut commands: Commands,
    mut reloaded: MessageReader<ConfigHotReloadedMessage>,
    asset_server: Res<AssetServer>,
    cfg: Res<VisualsConfig>,
    mut assets: ResMut<PlayerAnimAssets>,
    roots: Query<Entity, With<PlayerVisualRoot>>,
    mut model_children: Query<(Entity, &mut Transform), With<PlayerModelChild>>,
) {
    if reloaded.read().next().is_none() || !cfg.is_changed() {
        return;
    }

    if assets.loaded_path != cfg.player.model_path {
        load_player_gltf(&asset_server, &cfg.player.model_path, &mut assets);
        for (child_entity, _) in &model_children {
            commands.entity(child_entity).despawn();
        }
        for root_entity in &roots {
            commands.entity(root_entity).remove::<PlayerModelSpawned>();
        }
        return;
    }

    let child_tf = model_child_transform(&cfg.player);
    for (_, mut transform) in &mut model_children {
        *transform = child_tf;
    }
}

#[allow(clippy::too_many_arguments)]
pub fn sync_animation_catalog_hot_reload(
    mut reloaded: MessageReader<ConfigHotReloadedMessage>,
    catalog: Res<AnimCatalog>,
    gltfs: Res<Assets<Gltf>>,
    targets: Query<(Entity, &AnimationTargetId, &Name)>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
    mut assets: ResMut<PlayerAnimAssets>,
    mut players: Query<(
        &mut AnimationGraphHandle,
        &mut AnimationPlayer,
        &mut director::DirectorState,
        &mut PlayerRig,
    )>,
) {
    if reloaded.read().next().is_none() || !catalog.is_changed() {
        return;
    }
    if assets.graph.is_none() || assets.nodes.is_none() {
        return;
    }
    let Some(gltf_handle) = assets.gltf.clone() else {
        return;
    };
    let Some(gltf) = gltfs.get(&gltf_handle) else {
        return;
    };

    let target_list = collect_targets(&targets);
    let Some(handle) = rebuild_graph(gltf, &catalog, &target_list, &mut graphs, &mut assets) else {
        return;
    };
    let Some(nodes) = assets.nodes.clone() else {
        return;
    };
    let bones = rig::find_bones(
        &catalog.ik,
        targets
            .iter()
            .map(|(entity, _, name)| (entity, name.as_str())),
    );

    for (mut graph_handle, mut player, mut state, mut rig) in &mut players {
        player.stop_all();
        *state = director::DirectorState::default();
        rig.bones = bones;
        graph_handle.0 = handle.clone();
        director::prime_looped(&mut player, &nodes);
    }
}

pub fn prepare_player_model_assets(
    cfg: Res<VisualsConfig>,
    gltfs: Res<Assets<Gltf>>,
    mut assets: ResMut<PlayerAnimAssets>,
) {
    if assets.scene.is_some() {
        return;
    }
    let Some(gltf_handle) = assets.gltf.clone() else {
        return;
    };
    let Some(gltf) = gltfs.get(&gltf_handle) else {
        return;
    };

    assets.scene = gltf.scenes.first().cloned();
    if assets.scene.is_none() {
        warn!("Player glTF '{}' contains no scenes", cfg.player.model_path);
    }
}

pub fn attach_player_model_scene(
    mut commands: Commands,
    cfg: Res<VisualsConfig>,
    assets: Res<PlayerAnimAssets>,
    roots: Query<Entity, (With<PlayerVisualRoot>, Without<PlayerModelSpawned>)>,
) {
    let Some(scene_handle) = &assets.scene else {
        return;
    };

    for root_entity in &roots {
        commands
            .entity(root_entity)
            .insert(PlayerModelSpawned)
            .with_children(|parent| {
                parent.spawn((
                    Name::new("PlayerModelScene"),
                    PlayerModelChild,
                    WorldAssetRoot(scene_handle.clone()),
                    model_child_transform(&cfg.player),
                ));
            });
    }
}

#[allow(clippy::too_many_arguments)]
pub fn bind_player_animation_players(
    mut commands: Commands,
    catalog: Res<AnimCatalog>,
    gltfs: Res<Assets<Gltf>>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
    mut assets: ResMut<PlayerAnimAssets>,
    mut added_players: Query<(Entity, &ChildOf, &mut AnimationPlayer), Added<AnimationPlayer>>,
    parents: Query<&ChildOf>,
    visual_roots: Query<&ChildOf, With<PlayerVisualRoot>>,
    survivors: Query<(), With<Survivor>>,
    targets: Query<(Entity, &AnimationTargetId, &Name)>,
) {
    if added_players.is_empty() {
        return;
    }
    let Some(gltf_handle) = assets.gltf.clone() else {
        return;
    };
    let Some(gltf) = gltfs.get(&gltf_handle) else {
        return;
    };
    if let Err(err) = catalog.validate() {
        warn!("Animation catalog is invalid: {err}");
    }

    let nodes = if let Some(nodes) = assets.nodes.clone() {
        nodes
    } else {
        let target_list = collect_targets(&targets);
        if rebuild_graph(gltf, &catalog, &target_list, &mut graphs, &mut assets).is_none() {
            return;
        }
        let Some(nodes) = assets.nodes.clone() else {
            return;
        };
        nodes
    };
    let Some(graph_handle) = assets.graph.clone() else {
        return;
    };
    let bones = rig::find_bones(
        &catalog.ik,
        targets
            .iter()
            .map(|(entity, _, name)| (entity, name.as_str())),
    );

    for (anim_entity, first_parent, mut player) in &mut added_players {
        let Some((survivor, root)) =
            find_rig_owner(first_parent.parent(), &parents, &visual_roots, &survivors)
        else {
            continue;
        };
        commands.entity(anim_entity).insert((
            AnimationGraphHandle(graph_handle.clone()),
            PlayerRig {
                survivor,
                root,
                bones,
            },
            director::DirectorState::default(),
        ));
        director::prime_looped(&mut player, &nodes);
    }
}

fn collect_targets(
    targets: &Query<(Entity, &AnimationTargetId, &Name)>,
) -> Vec<(AnimationTargetId, String)> {
    targets
        .iter()
        .map(|(_, target, name)| (*target, name.as_str().to_string()))
        .collect()
}

fn rebuild_graph(
    gltf: &Gltf,
    catalog: &AnimCatalog,
    targets: &[(AnimationTargetId, String)],
    graphs: &mut Assets<AnimationGraph>,
    assets: &mut PlayerAnimAssets,
) -> Option<Handle<AnimationGraph>> {
    let result = graph::build(gltf, catalog, targets);
    if !result.missing_clips.is_empty() {
        let available: Vec<&str> = gltf
            .named_animations
            .keys()
            .map(|key| key.as_ref())
            .collect();
        warn!(
            "Animation roles missing from player glTF: {:?} (available: {:?})",
            result.missing_clips, available
        );
    }
    if !result.unmatched_mask_bones.is_empty() {
        warn!(
            "Animation mask bones not found on the player skeleton: {:?}",
            result.unmatched_mask_bones
        );
    }

    let handle = graphs.add(result.graph);
    assets.graph = Some(handle.clone());
    assets.nodes = Some(result.nodes);
    Some(handle)
}

pub fn disable_player_backface_culling(
    mut materials: ResMut<Assets<StandardMaterial>>,
    material_entities: Query<(Entity, &MeshMaterial3d<StandardMaterial>)>,
    parents: Query<&ChildOf>,
    roots: Query<(), With<PlayerVisualRoot>>,
) {
    for (entity, material_handle) in &material_entities {
        let mut cursor = entity;
        let mut is_player = false;
        for _ in 0..16 {
            if roots.contains(cursor) {
                is_player = true;
                break;
            }
            let Ok(parent) = parents.get(cursor) else {
                break;
            };
            cursor = parent.parent();
        }
        if !is_player {
            continue;
        }
        if let Some(mut material) = materials.get_mut(&material_handle.0) {
            material.cull_mode = None;
        }
    }
}

fn find_rig_owner(
    mut cursor: Entity,
    parents: &Query<&ChildOf>,
    visual_roots: &Query<&ChildOf, With<PlayerVisualRoot>>,
    survivors: &Query<(), With<Survivor>>,
) -> Option<(Entity, Entity)> {
    for _ in 0..8 {
        if let Ok(root_parent) = visual_roots.get(cursor) {
            let survivor = root_parent.parent();
            if survivors.contains(survivor) {
                return Some((survivor, cursor));
            }
            return None;
        }
        let Ok(next_parent) = parents.get(cursor) else {
            return None;
        };
        cursor = next_parent.parent();
    }
    None
}
