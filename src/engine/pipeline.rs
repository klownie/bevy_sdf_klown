use bevy::{
    pbr::{GpuClusterableObjectsStorage, GpuLights},
    prelude::*,
    render::{
        render_resource::{
            BindGroupLayout, BindGroupLayoutEntries, CachedComputePipelineId,
            ComputePipelineDescriptor, PipelineCache, ShaderStages, StorageTextureAccess,
            TextureFormat,
            binding_types::{
                storage_buffer_read_only, texture_depth_2d, texture_storage_2d, uniform_buffer,
            },
        },
        renderer::RenderDevice,
        view::ViewUniform,
    },
};

use super::{
    RAY_MARCH_MAIN_PASS_HANDLE, RayMarchCamera, op::SdOpUniformInstance,
    shape::SdShapeUniformInstance,
};

#[derive(Resource)]
pub struct RayMarchEnginePipeline {
    pub common_layout: BindGroupLayout,
    pub texture_layout: BindGroupLayout,
    pub storage_layout: BindGroupLayout,
    pub prepass_layout: BindGroupLayout,
    pub march_pipeline: CachedComputePipelineId,
    pub scale_pipeline: CachedComputePipelineId,
}

impl FromWorld for RayMarchEnginePipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let pipeline_cache = world.resource::<PipelineCache>();

        let common_layout = render_device.create_bind_group_layout(
            "ray_march_import_bind_group_layout",
            &BindGroupLayoutEntries::with_indices(
                ShaderStages::COMPUTE,
                (
                    (0, uniform_buffer::<ViewUniform>(true)),
                    // Directional Lights
                    (1, uniform_buffer::<GpuLights>(true)),
                    // Spotlights
                    (
                        8,
                        storage_buffer_read_only::<GpuClusterableObjectsStorage>(false),
                    ),
                ),
            ),
        );

        let texture_layout = render_device.create_bind_group_layout(
            "ray_march_texture_bind_group_layout",
            &BindGroupLayoutEntries::sequential(
                ShaderStages::COMPUTE,
                (
                    // texture_storage_2d(
                    //     ViewTarget::TEXTURE_FORMAT_HDR,
                    //     StorageTextureAccess::WriteOnly,
                    // ),
                    texture_depth_2d(),
                    uniform_buffer::<RayMarchCamera>(true),
                ),
            ),
        );

        let storage_layout = render_device.create_bind_group_layout(
            "ray_march_storage_bind_group_layout",
            &BindGroupLayoutEntries::sequential(
                ShaderStages::COMPUTE,
                (
                    storage_buffer_read_only::<SdShapeUniformInstance>(false),
                    storage_buffer_read_only::<SdOpUniformInstance>(false),
                ),
            ),
        );

        let prepass_layout = render_device.create_bind_group_layout(
            "ray_march_prepass_bind_group_layout",
            &BindGroupLayoutEntries::sequential(
                ShaderStages::COMPUTE,
                (
                    texture_storage_2d(TextureFormat::R32Float, StorageTextureAccess::WriteOnly),
                    texture_storage_2d(TextureFormat::Rgba32Float, StorageTextureAccess::WriteOnly),
                    texture_storage_2d(TextureFormat::Rgba32Float, StorageTextureAccess::WriteOnly),
                    texture_storage_2d(TextureFormat::R32Float, StorageTextureAccess::WriteOnly),
                    texture_storage_2d(TextureFormat::R32Float, StorageTextureAccess::ReadWrite),
                    texture_storage_2d(TextureFormat::Rgba32Float, StorageTextureAccess::ReadWrite),
                    texture_storage_2d(TextureFormat::Rgba32Float, StorageTextureAccess::ReadWrite),
                    texture_storage_2d(TextureFormat::R32Float, StorageTextureAccess::ReadWrite),
                ),
            ),
        );

        let march_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: Some("ray_march_pipeline".into()),
            layout: vec![
                common_layout.clone(),
                texture_layout.clone(),
                storage_layout.clone(),
                prepass_layout.clone(),
            ],
            shader: RAY_MARCH_MAIN_PASS_HANDLE,
            shader_defs: vec![],
            entry_point: "init".into(),
            push_constant_ranges: vec![],
            zero_initialize_workgroup_memory: false,
        });

        let scale_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: Some("ray_march_pipeline_2nd_pass".into()),
            layout: vec![
                common_layout.clone(),
                texture_layout.clone(),
                storage_layout.clone(),
                prepass_layout.clone(),
            ],
            shader: RAY_MARCH_MAIN_PASS_HANDLE,
            shader_defs: vec![],
            entry_point: "scale".into(),
            push_constant_ranges: vec![],
            zero_initialize_workgroup_memory: false,
        });

        Self {
            common_layout,
            texture_layout,
            storage_layout,
            prepass_layout,
            march_pipeline,
            scale_pipeline,
        }
    }
}
