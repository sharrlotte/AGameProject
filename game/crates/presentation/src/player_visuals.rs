use std::time::Duration;

use bevy::gltf::Gltf;
use bevy::prelude::*;
use bevy::world_serialization::{WorldAsset, WorldAssetRoot};
use bevy_rapier3d::prelude::Velocity;
use data::{AppRonConfigExt, ConfigHotReloadedMessage, GameState, SimUpdateSet};
use world::{Health, Stamina, Survivor};

use crate::config::{PlayerModelConfig, VisualsConfig};

/// Marker on the `VisualBody` pivot entity spawned as a direct child of `Survivor`.
#[derive(Component)]
pub struct PlayerVisualRoot;

#[derive(Component)]
struct PlayerModelChild;

#[derive(Component)]
struct PlayerModelSpawned;

/// Locomotion/death animation states supported by the player rig.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayerAnimClip {
    Idle,
    Walk,
    Sprint,
    Death,
}

#[derive(Component)]
struct PlayerAnimationController {
    survivor: Entity,
    current: Option<PlayerAnimClip>,
}

/// Cached glTF scene and animation graph handles built from `VisualsConfig`.
#[derive(Resource, Default)]
pub struct PlayerModelAssets {
    loaded_path: String,
    gltf: Option<Handle<Gltf>>,
    scene: Option<Handle<WorldAsset>>,
    graph: Option<Handle<AnimationGraph>>,
    idle_node: Option<AnimationNodeIndex>,
    walk_node: Option<AnimationNodeIndex>,
    run_node: Option<AnimationNodeIndex>,
    death_node: Option<AnimationNodeIndex>,
}

impl PlayerModelAssets {
    fn node_for(&self, clip: PlayerAnimClip) -> Option<AnimationNodeIndex> {
        match clip {
            PlayerAnimClip::Idle => self.idle_node,
            PlayerAnimClip::Walk => self.walk_node,
            PlayerAnimClip::Sprint => self.run_node,
            PlayerAnimClip::Death => self.death_node,
        }
    }

    fn clear_built(&mut self) {
        self.scene = None;
        self.graph = None;
        self.idle_node = None;
        self.walk_node = None;
        self.run_node = None;
        self.death_node = None;
    }
}

/// Registers systems that load, attach, hot-reload, and animate the player 3D glTF model.
pub struct PlayerVisualsPlugin;

impl Plugin for PlayerVisualsPlugin {
    fn build(&self, app: &mut App) {
        app.register_ron_config::<VisualsConfig>()
            .init_resource::<PlayerModelAssets>()
            .add_systems(OnExit(GameState::Loading), start_loading_player_model)
            .add_systems(
                Update,
                (
                    sync_player_visuals_hot_reload,
                    prepare_player_model_assets,
                    attach_player_model_scene,
                    bind_player_animation_players,
                )
                    .chain()
                    .after(SimUpdateSet)
                    .run_if(not(in_state(GameState::Loading))),
            )
            .add_systems(
                Update,
                update_player_animations
                    .after(bind_player_animation_players)
                    .run_if(|state: Res<State<GameState>>| {
                        matches!(state.get(), GameState::Playing | GameState::GameOver)
                    }),
            );
    }
}

fn start_loading_player_model(
    asset_server: Res<AssetServer>,
    cfg: Res<VisualsConfig>,
    mut model_assets: ResMut<PlayerModelAssets>,
) {
    load_player_gltf(&asset_server, &cfg.player.model_path, &mut model_assets);
}

fn load_player_gltf(
    asset_server: &AssetServer,
    model_path: &str,
    model_assets: &mut PlayerModelAssets,
) {
    model_assets.loaded_path = model_path.to_string();
    model_assets.gltf = Some(asset_server.load(model_path.to_string()));
    model_assets.clear_built();
}

fn model_child_transform(cfg: &PlayerModelConfig) -> Transform {
    Transform::from_xyz(0.0, cfg.y_offset, 0.0)
        .with_scale(Vec3::splat(cfg.scale))
        .with_rotation(Quat::from_rotation_y(cfg.yaw_offset_deg.to_radians()))
}

fn sync_player_visuals_hot_reload(
    mut commands: Commands,
    mut reloaded: MessageReader<ConfigHotReloadedMessage>,
    asset_server: Res<AssetServer>,
    cfg: Res<VisualsConfig>,
    gltfs: Res<Assets<Gltf>>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
    mut model_assets: ResMut<PlayerModelAssets>,
    roots: Query<Entity, With<PlayerVisualRoot>>,
    mut model_children: Query<(Entity, &mut Transform), With<PlayerModelChild>>,
    mut anim_players: Query<
        (&mut AnimationGraphHandle, &mut PlayerAnimationController),
        With<AnimationPlayer>,
    >,
) {
    if reloaded.read().next().is_none() || !cfg.is_changed() {
        return;
    }

    if model_assets.loaded_path != cfg.player.model_path {
        load_player_gltf(&asset_server, &cfg.player.model_path, &mut model_assets);
        for (child_entity, _) in &model_children {
            commands.entity(child_entity).despawn();
        }
        for root_entity in &roots {
            commands.entity(root_entity).remove::<PlayerModelSpawned>();
        }
        return;
    }

    let child_tf = model_child_transform(&cfg.player);
    for (_, mut tf) in &mut model_children {
        *tf = child_tf;
    }

    if let Some(gltf_handle) = &model_assets.gltf {
        if let Some(gltf) = gltfs.get(gltf_handle) {
            rebuild_animation_graph(gltf, &cfg.player, &mut graphs, &mut model_assets);
            if let Some(graph_handle) = &model_assets.graph {
                for (mut graph_comp, mut controller) in &mut anim_players {
                    graph_comp.0 = graph_handle.clone();
                    controller.current = None;
                }
            }
        }
    }
}

fn prepare_player_model_assets(
    cfg: Res<VisualsConfig>,
    gltfs: Res<Assets<Gltf>>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
    mut model_assets: ResMut<PlayerModelAssets>,
) {
    if model_assets.scene.is_some() && model_assets.graph.is_some() {
        return;
    }

    let Some(gltf_handle) = model_assets.gltf.clone() else {
        return;
    };
    let Some(gltf) = gltfs.get(&gltf_handle) else {
        return;
    };

    if model_assets.scene.is_none() {
        model_assets.scene = gltf.scenes.first().cloned();
        if model_assets.scene.is_none() {
            warn!("Player glTF '{}' contains no scenes", cfg.player.model_path);
        }
    }

    if model_assets.graph.is_none() {
        rebuild_animation_graph(gltf, &cfg.player, &mut graphs, &mut model_assets);
    }
}

fn rebuild_animation_graph(
    gltf: &Gltf,
    cfg: &PlayerModelConfig,
    graphs: &mut Assets<AnimationGraph>,
    model_assets: &mut PlayerModelAssets,
) {
    let mut graph = AnimationGraph::new();
    let idle_node = add_named_clip(&mut graph, gltf, &cfg.idle_clip);
    let walk_node = add_named_clip(&mut graph, gltf, &cfg.walk_clip);
    let run_node = add_named_clip(&mut graph, gltf, &cfg.run_clip);
    let death_node = add_named_clip(&mut graph, gltf, &cfg.death_clip);

    model_assets.idle_node = idle_node;
    model_assets.walk_node = walk_node;
    model_assets.run_node = run_node;
    model_assets.death_node = death_node;
    model_assets.graph = Some(graphs.add(graph));
}

fn add_named_clip(
    graph: &mut AnimationGraph,
    gltf: &Gltf,
    clip_name: &str,
) -> Option<AnimationNodeIndex> {
    if let Some(clip) = gltf.named_animations.get(clip_name) {
        Some(graph.add_clip(clip.clone(), 1.0, graph.root))
    } else {
        warn!(
            "Animation clip '{}' not found in player glTF (available: {:?})",
            clip_name,
            gltf.named_animations.keys().collect::<Vec<_>>()
        );
        None
    }
}

fn attach_player_model_scene(
    mut commands: Commands,
    cfg: Res<VisualsConfig>,
    model_assets: Res<PlayerModelAssets>,
    roots: Query<Entity, (With<PlayerVisualRoot>, Without<PlayerModelSpawned>)>,
) {
    let Some(scene_handle) = &model_assets.scene else {
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

fn bind_player_animation_players(
    mut commands: Commands,
    model_assets: Res<PlayerModelAssets>,
    added_players: Query<(Entity, &ChildOf), Added<AnimationPlayer>>,
    parents: Query<&ChildOf>,
    visual_roots: Query<&ChildOf, With<PlayerVisualRoot>>,
    survivors: Query<(), With<Survivor>>,
) {
    let Some(graph_handle) = &model_assets.graph else {
        return;
    };

    for (anim_entity, first_parent) in &added_players {
        if let Some(survivor) =
            find_survivor_ancestor(first_parent.parent(), &parents, &visual_roots, &survivors)
        {
            commands.entity(anim_entity).insert((
                AnimationGraphHandle(graph_handle.clone()),
                AnimationTransitions::new(),
                PlayerAnimationController {
                    survivor,
                    current: None,
                },
            ));
        }
    }
}

fn find_survivor_ancestor(
    mut cursor: Entity,
    parents: &Query<&ChildOf>,
    visual_roots: &Query<&ChildOf, With<PlayerVisualRoot>>,
    survivors: &Query<(), With<Survivor>>,
) -> Option<Entity> {
    for _ in 0..8 {
        if let Ok(root_parent) = visual_roots.get(cursor) {
            let candidate = root_parent.parent();
            if survivors.contains(candidate) {
                return Some(candidate);
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

fn update_player_animations(
    state: Res<State<GameState>>,
    cfg: Res<VisualsConfig>,
    model_assets: Res<PlayerModelAssets>,
    survivors: Query<(&Health, &Stamina, &Velocity), With<Survivor>>,
    mut anim_players: Query<(
        &mut AnimationPlayer,
        &mut AnimationTransitions,
        &mut PlayerAnimationController,
    )>,
) {
    let blend = Duration::from_secs_f32(cfg.player.blend_secs.max(0.0));
    let is_game_over = *state.get() == GameState::GameOver;

    for (mut player, mut transitions, mut controller) in &mut anim_players {
        let Ok((health, stamina, vel)) = survivors.get(controller.survivor) else {
            continue;
        };

        let horiz_speed = Vec2::new(vel.linear.x, vel.linear.z).length();
        let (target_clip, target_speed) = select_player_anim_clip(
            health.current > 0.0 && !is_game_over,
            horiz_speed,
            stamina.is_sprinting,
            &cfg.player,
        );

        let Some(node_idx) = model_assets.node_for(target_clip) else {
            continue;
        };

        if controller.current != Some(target_clip) {
            let active = transitions.play(&mut player, node_idx, blend);
            if target_clip != PlayerAnimClip::Death {
                active.repeat();
            }
            active.set_speed(target_speed);
            controller.current = Some(target_clip);
        } else if let Some(active) = player.animation_mut(node_idx) {
            active.set_speed(target_speed);
        }
    }
}

/// Computes the target clip and playback speed for the survivor's current state.
pub fn select_player_anim_clip(
    alive: bool,
    horizontal_speed: f32,
    is_sprinting: bool,
    cfg: &PlayerModelConfig,
) -> (PlayerAnimClip, f32) {
    if !alive {
        return (PlayerAnimClip::Death, 1.0);
    }
    if horizontal_speed > cfg.move_speed_threshold {
        if is_sprinting {
            return (PlayerAnimClip::Sprint, cfg.sprint_playback_speed.max(0.1));
        }
        let walk_scale = (horizontal_speed / cfg.walk_speed_ref.max(0.1)).clamp(0.6, 1.6);
        return (PlayerAnimClip::Walk, walk_scale);
    }
    (PlayerAnimClip::Idle, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn select_clip_transitions_between_idle_walk_sprint_and_death() {
        let cfg = PlayerModelConfig::default();

        assert_eq!(
            select_player_anim_clip(true, 0.0, false, &cfg),
            (PlayerAnimClip::Idle, 1.0)
        );
        assert_eq!(
            select_player_anim_clip(true, cfg.walk_speed_ref, false, &cfg),
            (PlayerAnimClip::Walk, 1.0)
        );
        assert_eq!(
            select_player_anim_clip(true, 11.0, true, &cfg),
            (PlayerAnimClip::Sprint, cfg.sprint_playback_speed)
        );
        assert_eq!(
            select_player_anim_clip(false, 11.0, true, &cfg),
            (PlayerAnimClip::Death, 1.0)
        );
    }
}
