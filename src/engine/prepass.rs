use bevy::{prelude::*, render::render_resource::TextureView};

#[derive(Component)]
pub struct RayMarchPrepass {
    pub depth: TextureView,
    pub normal: TextureView,
    pub output: TextureView,
    pub view_size: UVec2,
}
