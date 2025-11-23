use bevy::{
    prelude::*,
    render::{extract_resource::ExtractResource, render_resource::Buffer},
};

#[derive(Resource, Clone, ExtractResource)]
pub struct RayMarchBuffer {
    pub object: Buffer,
    pub operator: Buffer,
    pub modifier: Buffer,
    pub field_data: Buffer,
}
