#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput

@group(0) @binding(0) var raymarch_depth_texture: texture_2d<f32>;
@group(0) @binding(1) var raymarch_output_texture: texture_2d<f32>;
@group(0) @binding(2) var texture_sampler: sampler;

struct FragmentOutput {
    @location(0) color: vec4f,
    @builtin(frag_depth) frag_depth: f32,
};

@fragment
fn fragment(in: FullscreenVertexOutput) -> FragmentOutput {

    let raymarch_depth = textureSample(raymarch_depth_texture, texture_sampler, in.uv).x;
    let raymarch_output = textureSample(raymarch_output_texture, texture_sampler, in.uv);

    return FragmentOutput(
        raymarch_output, 
        raymarch_depth
    );
}
