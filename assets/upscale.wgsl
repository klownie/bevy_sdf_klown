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

@group(1) @binding(0) var march_texture: texture_2d<f32>;

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
  
    // var uv = (in.uv - 0.5) / settings.down_scale + 0.5;

    let uv = in.uv;
    
    let color = textureSample(march_texture, texture_sampler, uv);
    return vec4f(color.rgb, 1.0);
}
