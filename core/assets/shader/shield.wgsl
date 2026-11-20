#import bevy_sprite::mesh2d_vertex_output::Mesh2dVertexOutput

struct ShieldMaterial {
    color: vec4<f32>,
    hp_ratio: f32,
    time: f32,
    _padding: vec2<f32>,
};

@group(2) @binding(0)
var<uniform> material: ShieldMaterial;

@fragment
fn fragment(mesh: Mesh2dVertexOutput) -> @location(0) vec4<f32> {
    let uv = mesh.uv;
    let center_dist = distance(uv, vec2<f32>(0.5, 0.5)) * 2.0;
    let pulse = 0.85 + 0.15 * sin(material.time * 3.0);
    let edge = pow(center_dist, 2.0);
    let alpha = clamp(material.color.a * material.hp_ratio * (0.2 + 0.8 * edge) * pulse, 0.0, 1.0);
    return vec4<f32>(material.color.rgb * 1.5, alpha);
}
