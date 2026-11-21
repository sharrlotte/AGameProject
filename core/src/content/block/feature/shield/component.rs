use bevy::prelude::*;
use bevy::render::render_resource::AsBindGroup;
use bevy::shader::ShaderRef;
use bevy::sprite_render::{AlphaMode2d, Material2d};

use crate::config::GameConfig;
use crate::content::block::feature::power::{Consumer, Demand, Supply};
use crate::content::team::Team;

#[derive(Clone, Copy, Debug, PartialEq, Reflect)]
pub enum ShieldShape {
    Circle {
        radius: f32,
    },
    Plane {
        width: f32,
        thickness: f32,
        offset: Vec2,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Reflect)]
pub enum ShieldState {
    Active,
    Depleted { cooldown_remaining: f32 },
    Regenerating,
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct ShieldMaterial {
    #[uniform(0)]
    pub color: LinearRgba,
    #[uniform(0)]
    pub hp_ratio: f32,
    #[uniform(0)]
    pub time: f32,
    #[uniform(0)]
    pub shape_type: f32,
    #[uniform(0)]
    pub padding: f32,
}

impl Material2d for ShieldMaterial {
    fn fragment_shader() -> ShaderRef {
        "shader/shield.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }
}

#[derive(Component, Clone, PartialEq, Reflect)]
pub struct Shield {
    pub shape: ShieldShape,
    pub max_hp: f32,
    pub cooldown: f32,
    pub regen_rate: f32,
    pub demand: Option<f32>,
    pub state: ShieldState,
    pub barrier: Option<Entity>,
}

impl Shield {
    pub fn new(
        shape: ShieldShape,
        max_hp: f32,
        cooldown: f32,
        regen_rate: f32,
        demand: Option<f32>,
    ) -> Self {
        Self {
            shape,
            max_hp,
            cooldown,
            regen_rate,
            demand,
            state: ShieldState::Active,
            barrier: None,
        }
    }

    pub fn insert(
        &self,
        commands: &mut Commands,
        base: Entity,
        _visual: Entity,
        _pixel_size: Vec2,
        _team: &Team,
        _config: &GameConfig,
    ) {
        commands.entity(base).insert(self.clone());
        
        if let Some(rate) = self.demand {
            if rate > 0.0 {
                commands.entity(base).insert((
                    Consumer { rate },
                    Demand { value: rate },
                    Supply::default(),
                ));
            }
        }
    }
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Reflect)]
pub struct Barrier {
    pub emitter: Entity,
    pub shape: ShieldShape,
}
