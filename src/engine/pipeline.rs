use bevy::{
    pbr::{GpuClusterableObjectsStorage, GpuLights},
    prelude::*,
    render::{
        render_resource::{
            BindGroupLayout, BindGroupLayoutEntries, ComputePipelineDescriptor, Sampler,
            SamplerBindingType, SamplerDescriptor, ShaderStages, SpecializedComputePipeline,
            StorageTextureAccess, TextureFormat,
            binding_types::{
                sampler, storage_buffer_read_only, texture_2d, texture_depth_2d,
                texture_storage_2d, uniform_buffer,
            },
        },
        renderer::RenderDevice,
        view::{ViewTarget, ViewUniform},
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
}

impl FromWorld for RayMarchEnginePipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();

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
                (texture_depth_2d(), uniform_buffer::<RayMarchCamera>(true)),
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
            "ray_march_storage_bind_group_layout",
            &BindGroupLayoutEntries::sequential(
                ShaderStages::COMPUTE,
                (
                    texture_storage_2d(TextureFormat::R32Float, StorageTextureAccess::WriteOnly),
                    texture_storage_2d(TextureFormat::Rgba32Float, StorageTextureAccess::WriteOnly),
                    texture_storage_2d(TextureFormat::Rgba32Float, StorageTextureAccess::WriteOnly),
                    texture_storage_2d(TextureFormat::R32Float, StorageTextureAccess::WriteOnly),
                ),
            ),
        );

        Self {
            common_layout,
            texture_layout,
            storage_layout,
            prepass_layout,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct RayMarchEnginePipelineKey {
    pub hdr: bool,
}

impl SpecializedComputePipeline for RayMarchEnginePipeline {
    type Key = RayMarchEnginePipelineKey;

    fn specialize(&self, key: Self::Key) -> ComputePipelineDescriptor {
        let format = if key.hdr {
            ViewTarget::TEXTURE_FORMAT_HDR
        } else {
            TextureFormat::bevy_default()
        };

        ComputePipelineDescriptor {
            label: Some("ray_march_pipeline".into()),
            layout: vec![
                self.common_layout.clone(),
                self.texture_layout.clone(),
                self.storage_layout.clone(),
                self.prepass_layout.clone(),
            ],
            shader: RAY_MARCH_MAIN_PASS_HANDLE,
            shader_defs: vec![],
            entry_point: "init".into(),
            push_constant_ranges: vec![],
            zero_initialize_workgroup_memory: false,
        }
    }
}
