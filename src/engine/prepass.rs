use bevy::{
    prelude::*,
    render::{
        camera::ExtractedCamera,
        render_resource::{
            Extent3d, TextureAspect, TextureDescriptor, TextureDimension, TextureFormat,
            TextureUsages, TextureView, TextureViewDescriptor,
        },
        renderer::RenderDevice,
    },
};

use super::camera::RayMarchCamera;

#[derive(Component)]
pub struct RayMarchPrepass {
    pub depth: TextureView,
    pub normal: TextureView,
    pub material: TextureView,
    pub mask: TextureView,
    pub scaled_depth: TextureView,
    pub scaled_normal: TextureView,
    pub scaled_material: TextureView,
    pub scaled_mask: TextureView,
    pub view_size: UVec2,
}

pub fn prepare_ray_march_resources(
    query: Query<(Entity, &ExtractedCamera, Option<&RayMarchPrepass>), With<RayMarchCamera>>,
    render_device: Res<RenderDevice>,
    mut commands: Commands,
) {
    for (entity, camera, ray_march_resources) in &query {
        let Some(view_size) = camera.physical_viewport_size else {
            continue;
        };

        if ray_march_resources.map(|r| r.view_size) == Some(view_size) {
            continue;
        }

        let size = Extent3d {
            width: view_size.x,
            height: view_size.y,
            depth_or_array_layers: 1,
        };

        let depth = render_device
            .create_texture(&TextureDescriptor {
                label: Some("raymarch_depth"),
                size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: TextureFormat::R16Float,
                usage: TextureUsages::COPY_DST
                    | TextureUsages::STORAGE_BINDING
                    | TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            })
            .create_view(&TextureViewDescriptor {
                label: Some("raymarch_depth_view"),
                base_mip_level: 0,
                aspect: TextureAspect::All,
                base_array_layer: 0,
                ..default()
            });

        let normal = render_device
            .create_texture(&TextureDescriptor {
                label: Some("raymarch_normal"),
                size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: TextureFormat::Rgba16Float,
                usage: TextureUsages::COPY_DST
                    | TextureUsages::STORAGE_BINDING
                    | TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            })
            .create_view(&TextureViewDescriptor {
                label: Some("raymarch_normal_view"),
                base_mip_level: 0,
                aspect: TextureAspect::All,
                base_array_layer: 0,
                ..default()
            });

        let material = render_device
            .create_texture(&TextureDescriptor {
                label: Some("raymarch_material"),
                size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: TextureFormat::Rgba16Float,
                usage: TextureUsages::COPY_DST
                    | TextureUsages::STORAGE_BINDING
                    | TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            })
            .create_view(&TextureViewDescriptor {
                label: Some("raymarch_material_view"),
                base_mip_level: 0,
                aspect: TextureAspect::All,
                base_array_layer: 0,
                ..default()
            });

        let mask = render_device
            .create_texture(&TextureDescriptor {
                label: Some("raymarch_mask"),
                size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: TextureFormat::R16Float,
                usage: TextureUsages::COPY_DST
                    | TextureUsages::STORAGE_BINDING
                    | TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            })
            .create_view(&TextureViewDescriptor {
                label: Some("raymarch_mask_view"),
                base_mip_level: 0,
                aspect: TextureAspect::All,
                base_array_layer: 0,
                ..default()
            });

        let scaled_depth = render_device
            .create_texture(&TextureDescriptor {
                label: Some("scaled_raymarch_depth"),
                size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: TextureFormat::R16Float,
                usage: TextureUsages::COPY_DST
                    | TextureUsages::STORAGE_BINDING
                    | TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            })
            .create_view(&TextureViewDescriptor {
                label: Some("scaled_raymarch_depth_view"),
                base_mip_level: 0,
                aspect: TextureAspect::All,
                base_array_layer: 0,
                ..default()
            });

        let scaled_normal = render_device
            .create_texture(&TextureDescriptor {
                label: Some("scaled_raymarch_normal"),
                size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: TextureFormat::Rgba16Float,
                usage: TextureUsages::COPY_DST
                    | TextureUsages::STORAGE_BINDING
                    | TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            })
            .create_view(&TextureViewDescriptor {
                label: Some("scaled_raymarch_normal_view"),
                base_mip_level: 0,
                aspect: TextureAspect::All,
                base_array_layer: 0,
                ..default()
            });

        let scaled_material = render_device
            .create_texture(&TextureDescriptor {
                label: Some("scaled_raymarch_material"),
                size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: TextureFormat::Rgba16Float,
                usage: TextureUsages::COPY_DST
                    | TextureUsages::STORAGE_BINDING
                    | TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            })
            .create_view(&TextureViewDescriptor {
                label: Some("scaled_raymarch_material_view"),
                base_mip_level: 0,
                aspect: TextureAspect::All,
                base_array_layer: 0,
                ..default()
            });

        let scaled_mask = render_device
            .create_texture(&TextureDescriptor {
                label: Some("scaled_raymarch_mask"),
                size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: TextureFormat::R16Float,
                usage: TextureUsages::COPY_DST
                    | TextureUsages::STORAGE_BINDING
                    | TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            })
            .create_view(&TextureViewDescriptor {
                label: Some("scaled_raymarch_mask_view"),
                base_mip_level: 0,
                aspect: TextureAspect::All,
                base_array_layer: 0,
                ..default()
            });

        commands.entity(entity).insert(RayMarchPrepass {
            depth,
            normal,
            material,
            mask,
            scaled_depth,
            scaled_normal,
            scaled_material,
            scaled_mask,
            view_size,
        });
    }
}
