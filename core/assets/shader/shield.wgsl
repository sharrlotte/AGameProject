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

// Computes 2D regular hexagon tiling
// Returns vec3(offset_in_cell.x, offset_in_cell.y, edge_distance)
fn hex_coords(uv: vec2<f32>) -> vec3<f32> {
    let s = vec2<f32>(1.0, 1.7320508);
    let h = s * 0.5;
    let a = uv - s * round(uv / s);
    let b = (uv - h) - s * round((uv - h) / s);

    var g = a;
    if dot(a, a) > dot(b, b) {
        g = b;
    }

    let q = abs(g);
    let edge_dist = 0.5 - max(dot(q, vec2<f32>(0.5, 0.8660254)), q.x);
    return vec3<f32>(g.x, g.y, edge_dist);
}

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    let p = (mesh.uv - vec2<f32>(0.5, 0.5)) * 2.0;
    let instability = clamp(1.0 - material.hp_ratio, 0.0, 1.0);
    let flicker = 1.0 - instability * 0.35 * abs(sin(material.time * 28.0));

    if material.shape_type < 0.5 {
        // --- CIRCLE SHIELD ---
        let dist = length(p);
        if dist > 1.0 {
            discard;
        }

        // 1. Boundary rim aligned exactly at collision radius 1.0
        let rim_target = 0.985;
        let rim = smoothstep(0.035, 0.0, abs(dist - rim_target));

        // 2. Full Field Hex Grid wireframe (zero fill inside cells)
        let hex = hex_coords(p * 14.0);
        let hex_wire = smoothstep(0.06, 0.015, hex.z);

        // 3. Traveling radial energy ripple across the honeycomb
        let ripple = 0.5 + 0.5 * sin(dist * 18.0 - material.time * 3.5);
        let hex_intensity = hex_wire * (0.35 + 0.45 * ripple);

        // Combine rim + hex wireframe (no interior fog; background stays clear)
        let combined = max(rim * 0.95, hex_intensity);
        let alpha = clamp(
            combined * material.color.a * material.hp_ratio * flicker,
            0.0,
            1.0
        );

        let color_boost = 1.0 + rim * 0.8;
        return vec4<f32>(material.color.rgb * color_boost, alpha);
    } else {
        // --- PLANE SHIELD ---
        let target_y = 0.96;
        if p.y > 1.0 || p.y < -0.95 || abs(p.x) > 1.0 {
            discard;
        }

        let rim = smoothstep(0.06, 0.0, abs(p.y - target_y));
        let hex = hex_coords(p * 12.0);
        let hex_wire = smoothstep(0.06, 0.015, hex.z);
        let ripple = 0.5 + 0.5 * sin(p.y * 14.0 - material.time * 4.0);
        let hex_intensity = hex_wire * (0.3 + 0.4 * ripple);

        let side_fade = 1.0 - smoothstep(0.85, 1.0, abs(p.x));
        let combined = max(rim * 0.95, hex_intensity);
        let alpha = clamp(
            combined * side_fade * material.color.a * material.hp_ratio * flicker,
            0.0,
            1.0
        );

        let color_boost = 1.0 + rim * 0.8;
        return vec4<f32>(material.color.rgb * color_boost, alpha);
    }
}
