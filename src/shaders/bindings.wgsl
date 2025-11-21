#define_import_path bevy_sdf::bindings

#import bevy_sdf::types::{
    // SDF Object-related
    SdObjectPacked,
    SdOperatorPacked,
    SdMod
}

@group(1) @binding(0) var screen_texture: texture_storage_2d<rgba16float, write>;
@group(1) @binding(1) var depth_texture: texture_depth_2d;
struct RayMarchCamera {
    depth_scale: f32,
    eps: f32,
    w: f32,
    max_distance: f32,
    max_steps: u32,
    shadow_eps: f32,
    shadow_max_steps: u32,
    shadow_max_distance: f32,
    shadow_softness: f32,
    normal_eps: f32
}
@group(1) @binding(2) var<uniform> settings: RayMarchCamera;

// PERF: seperate the sd_object buffer into multiple buffers for more performance
@group(2) @binding(0) var<storage, read> sd_object: array<SdObjectPacked>;
@group(2) @binding(1) var<storage, read> sd_ops: array<SdOperatorPacked>;
@group(2) @binding(2) var<storage, read> sd_mod: array<SdMod>;
//@group(2) @binding(3) var<storage, read> sd_data: array<f32>;

@group(3) @binding(0) var depth_prepass: texture_storage_2d<r32float, read_write>;
@group(3) @binding(1) var normal_prepass: texture_storage_2d<rgba16float, write>;
@group(3) @binding(2) var material_prepass: texture_storage_2d<rgba16float, read_write>;
