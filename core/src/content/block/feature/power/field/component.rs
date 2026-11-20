use bevy::{
    color::LinearRgba,
    math::{Vec3, Vec4},
    prelude::*,
    render::render_resource::{AsBindGroup, ShaderType},
    shader::ShaderRef,
    sprite_render::{AlphaMode2d, Material2d},
};

pub const OVERDRIVE_FILL_COLOR: LinearRgba = LinearRgba::new(1.0, 0.45, 0.0, 0.12);
pub const OVERDRIVE_RING_COLOR: LinearRgba = LinearRgba::new(1.0, 0.55, 0.0, 0.75);
pub const OVERDRIVE_RING_WIDTH: f32 = 4.0;

#[derive(Component, Clone, Copy, Debug, Default, Reflect)]
#[reflect(Component)]
pub struct Field;

#[derive(Clone, Copy, Default, ShaderType)]
pub struct OverdriveMaterialUniform {
    pub color: Vec4,
    pub border_color: Vec4,
    pub ring_ratio: f32,
    pub _padding: Vec3,
}

#[derive(Asset, AsBindGroup, TypePath, Debug, Clone)]
#[uniform(0, OverdriveMaterialUniform)]
pub struct OverdriveMaterial {
    pub color: LinearRgba,
    pub border_color: LinearRgba,
    pub ring_ratio: f32,
}

impl From<&OverdriveMaterial> for OverdriveMaterialUniform {
    fn from(material: &OverdriveMaterial) -> Self {
        Self {
            color: material.color.to_f32_array().into(),
            border_color: material.border_color.to_f32_array().into(),
            ring_ratio: material.ring_ratio,
            _padding: Vec3::ZERO,
        }
    }
}

impl Material2d for OverdriveMaterial {
    fn fragment_shader() -> ShaderRef {
        "shader/overdrive_field.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }
}

#[derive(Resource, Clone)]
pub struct FieldAssets {
    pub mesh: Handle<Mesh>,
    pub material: Handle<OverdriveMaterial>,
}
