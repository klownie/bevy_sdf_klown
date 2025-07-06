use bevy::{
    prelude::*,
    render::{extract_component::ExtractComponent, render_resource::ShaderType},
};

#[derive(Component, Clone, Reflect, ExtractComponent, ShaderType)]
#[reflect(Component, Default)]
pub struct RayMarchCamera {
    pub depth_scale: f32,
    pub eps: f32,
    pub w: f32,
    pub max_distance: f32,
    pub max_steps: u32,
    pub shadow_eps: f32,
    pub shadow_max_steps: u32,
    pub shadow_max_distance: f32,
    pub shadow_softness: f32,
    pub normal_eps: f32,
}

impl Default for RayMarchCamera {
    fn default() -> Self {
        Self {
            depth_scale: 0.3,
            eps: 0.007,
            w: 1.,
            max_steps: 500,
            max_distance: 500.,
            shadow_eps: 0.1,
            shadow_max_steps: 500,
            shadow_max_distance: 100.,
            shadow_softness: 0.02,
            normal_eps: 0.01,
        }
    }
}
