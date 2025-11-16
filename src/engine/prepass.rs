use bevy::{prelude::*, render::render_resource::TextureView};

#[derive(Component)]
pub struct RayMarchPrepass {
    pub depth: TextureView,
    pub normal: TextureView,
    pub scaled_depth: TextureView,
    pub scaled_normal: TextureView,
    pub scaled_material: TextureView,
    pub view_size: UVec2,
}
