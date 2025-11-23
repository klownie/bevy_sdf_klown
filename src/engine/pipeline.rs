use std::borrow::Cow;

use bevy::{
    pbr::{GpuClusterableObjectsStorage, GpuLights},
    prelude::*,
    render::{
        render_resource::{
            BindGroupLayout, BindGroupLayoutEntries, CachedComputePipelineId,
            ComputePipelineDescriptor, PipelineCache, ShaderStages, StorageTextureAccess,
            TextureFormat,
            binding_types::{
                storage_buffer_read_only, storage_buffer_read_only_sized, texture_depth_2d,
                texture_storage_2d, uniform_buffer,
            },
        },
        renderer::RenderDevice,
        view::ViewUniform,
    },
};

use super::{RAY_MARCH_COMPUTE_PASS_HANDLE, RayMarchCamera};

#[derive(Resource)]
pub struct RayMarchEnginePipeline {
    pub common_layout: BindGroupLayout,
    pub texture_layout: BindGroupLayout,
    pub storage_layout: BindGroupLayout,
    pub prepass_layout: BindGroupLayout,
    pub compute_raymarch_pipeline: CachedComputePipelineId,
    pub compute_mask_pipeline: CachedComputePipelineId,
}

pub(crate) fn init_raymarch_compute_pipeline(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    pipeline_cache: Res<PipelineCache>,
) {
    let common_layout = render_device.create_bind_group_layout(
        "raymarch_import_bind_group_layout",
        &BindGroupLayoutEntries::with_indices(
            ShaderStages::COMPUTE,
            (
                (0, uniform_buffer::<ViewUniform>(true)),
                (1, uniform_buffer::<GpuLights>(true)),
                (
                    8,
                    storage_buffer_read_only::<GpuClusterableObjectsStorage>(false),
                ),
            ),
        ),
    );

    let texture_layout = render_device.create_bind_group_layout(
        "raymarch_texture_bind_group_layout",
        &BindGroupLayoutEntries::sequential(
            ShaderStages::COMPUTE,
            (texture_depth_2d(), uniform_buffer::<RayMarchCamera>(true)),
        ),
    );

    let storage_layout = render_device.create_bind_group_layout(
        "raymarch_storage_bind_group_layout",
        &BindGroupLayoutEntries::sequential(
            ShaderStages::COMPUTE,
            (
                storage_buffer_read_only_sized(false, None),
                storage_buffer_read_only_sized(false, None),
                storage_buffer_read_only_sized(false, None),
                storage_buffer_read_only_sized(false, None),
            ),
        ),
    );

    let prepass_layout = render_device.create_bind_group_layout(
        "raymarch_prepass_bind_group_layout",
        &BindGroupLayoutEntries::sequential(
            ShaderStages::COMPUTE,
            (
                texture_storage_2d(TextureFormat::R32Float, StorageTextureAccess::ReadWrite),
                texture_storage_2d(TextureFormat::Rgba16Float, StorageTextureAccess::WriteOnly),
                texture_storage_2d(TextureFormat::R16Float, StorageTextureAccess::WriteOnly),
                texture_storage_2d(TextureFormat::Rgba16Float, StorageTextureAccess::ReadWrite),
            ),
        ),
    );

    let march_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
        label: Some("raymarch_pipeline_compute_raymarch_pass".into()),
        layout: vec![
            common_layout.clone(),
            texture_layout.clone(),
            storage_layout.clone(),
            prepass_layout.clone(),
        ],
        shader: RAY_MARCH_COMPUTE_PASS_HANDLE,
        entry_point: Some(Cow::from("compute_raymarch")),
        ..default()
    });

    let scale_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
        label: Some("raymarch_pipeline_compute_mask_pass".into()),
        layout: vec![
            common_layout.clone(),
            texture_layout.clone(),
            storage_layout.clone(),
            prepass_layout.clone(),
        ],
        shader: RAY_MARCH_COMPUTE_PASS_HANDLE,
        entry_point: Some(Cow::from("compute_mask")),
        ..default()
    });

    commands.insert_resource(RayMarchEnginePipeline {
        common_layout,
        texture_layout,
        storage_layout,
        prepass_layout,
        compute_raymarch_pipeline: march_pipeline,
        compute_mask_pipeline: scale_pipeline,
    });
}
