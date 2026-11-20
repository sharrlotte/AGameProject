struct OverdriveMaterial {
    color: vec4<f32>,
    border_color: vec4<f32>,
    ring_ratio: f32,
    radius: f32,
    _padding: vec2<f32>,
};

@group(2) @binding(0) var<uniform> material: OverdriveMaterial;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) world_position: vec4<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
};

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    // uv ranges from 0.0 to 1.0; compute offset from center (-1.0 to 1.0)
    let p = (mesh.uv - vec2<f32>(0.5, 0.5)) * 2.0;
    let dist = length(p);

    // Discard outside the outer boundary with smooth anti-aliased edge
    let edge_aa = fwidth(dist);
    let outer_alpha = 1.0 - smoothstep(1.0 - edge_aa, 1.0 + edge_aa, dist);
    if outer_alpha <= 0.0 {
        discard;
    }

    // Inner ring threshold: 1.0 - ring_ratio. Discard inside (no fill)
    let ring_threshold = max(0.0, 1.0 - material.ring_ratio);
    let inner_alpha = smoothstep(ring_threshold - edge_aa, ring_threshold + edge_aa, dist);
    if inner_alpha <= 0.0 {
        discard;
    }

    // Static dashed pattern along circumference proportional to radius
    let angle = atan2(p.y, p.x);
    let PI = 3.14159265359;
    let normalized_angle = (angle + PI) / (2.0 * PI);
    // Period is ~24 pixels along circumference (12px dash, 12px gap)
    let circumference = 2.0 * PI * max(1.0, material.radius);
    let dash_count = max(8.0, round(circumference / 24.0));
    let dash_pos = fract(normalized_angle * dash_count);
    if dash_pos > 0.5 {
        discard;
    }

    let alpha = inner_alpha * outer_alpha * material.border_color.a;
    return vec4<f32>(material.border_color.rgb, alpha);
}
