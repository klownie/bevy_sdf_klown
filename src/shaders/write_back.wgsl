#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput

@group(0) @binding(0) var screen_texture: texture_2d<f32>;
@group(0) @binding(1) var texture_sampler: sampler;

struct RayMarchCamera {
    down_scale: f32,
    eps: f32,
    max_distance: f32,
    max_steps: u32,
}

@group(0) @binding(2) var<uniform> settings: RayMarchCamera;

@group(1) @binding(0) var march_material_texture: texture_2d<f32>;
@group(1) @binding(1) var march_mask_texture: texture_2d<f32>;

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4f {
    let uv = in.uv;

    let mask = bool(textureSample(march_mask_texture, texture_sampler, uv).x);

    let mat = textureSample(march_material_texture, texture_sampler, uv);

    let bg = textureSample(screen_texture, texture_sampler, uv);

    return select(bg, mat, mask);
}
