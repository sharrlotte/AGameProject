struct ShieldMaterial {
    color: vec4<f32>,
    hp_ratio: f32,
    time: f32,
    shape_type: f32,
    padding: f32,
};

@group(2) @binding(0)
var<uniform> material: ShieldMaterial;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) world_position: vec4<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
};

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    let p = (mesh.uv - vec2<f32>(0.5, 0.5)) * 2.0;
    let instability = clamp(1.0 - material.hp_ratio, 0.0, 1.0);
    let flicker = 1.0 - instability * 0.25 * abs(sin(material.time * 28.0));

    if material.shape_type < 0.5 {
        // Circle Shield: Rotating harmonic plasma wave + electric jitter
        let dist = length(p);
        let angle = atan2(p.y, p.x);

        let jitter = (0.005 + instability * 0.018) * sin(72.0 * angle + material.time * 20.0);
        let wave = 0.035 * sin(18.0 * angle + material.time * 2.4)
                 + 0.015 * cos(9.0 * angle - material.time * 1.6)
                 + jitter;
        let target_r = 0.94 + wave;

        let edge_aa = fwidth(dist);
        let outer_alpha = 1.0 - smoothstep(target_r - edge_aa, target_r + 0.04 + edge_aa, dist);
        if outer_alpha <= 0.0 {
            discard;
        }

        // Glowing perimeter filament line
        let filament = smoothstep(0.05, 0.0, abs(dist - target_r));
        // Subtle tinted interior fill (~5% opacity)
        let fill = smoothstep(target_r, 0.0, dist) * 0.05;

        let alpha = clamp(
            (filament * 0.95 + fill) * outer_alpha * material.color.a * material.hp_ratio * flicker,
            0.0,
            1.0
        );

        let color_boost = 1.0 + filament * 0.75;
        return vec4<f32>(material.color.rgb * color_boost, alpha);
    } else {
        // Plane Shield: Horizontal traveling deflection wave along front edge
        let jitter = (0.01 + instability * 0.02) * sin(p.x * 48.0 + material.time * 22.0);
        let wave = 0.08 * sin(p.x * 12.0 + material.time * 3.5) + jitter;
        let target_y = 0.78 + wave;

        let outer_alpha = 1.0 - smoothstep(target_y - 0.02, target_y + 0.12, p.y);
        if outer_alpha <= 0.0 || p.y < -0.95 {
            discard;
        }

        let side_fade = 1.0 - smoothstep(0.85, 1.0, abs(p.x));
        let filament = smoothstep(0.18, 0.0, abs(p.y - target_y));
        let fill = smoothstep(target_y, -1.0, p.y) * 0.05;

        let alpha = clamp(
            (filament * 0.95 + fill) * outer_alpha * side_fade * material.color.a * material.hp_ratio * flicker,
            0.0,
            1.0
        );

        let color_boost = 1.0 + filament * 0.75;
        return vec4<f32>(material.color.rgb * color_boost, alpha);
    }
}
