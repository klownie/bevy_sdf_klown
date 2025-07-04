use bevy::ecs::query::QueryItem;
use bevy::pbr::{GlobalClusterableObjectMeta, LightMeta};
use bevy::prelude::*;
use bevy::render::camera::ExtractedCamera;
use bevy::render::extract_component::ComponentUniforms;
use bevy::render::render_asset::RenderAssets;
use bevy::render::render_graph::NodeRunError;
use bevy::render::render_resource::{
    BindGroupEntries, BufferUsages, BufferVec, ComputePassDescriptor, PipelineCache,
};
use bevy::render::renderer::{RenderContext, RenderDevice, RenderQueue};
use bevy::render::texture::GpuImage;
use bevy::render::view::ViewUniforms;
use bevy::{
    core_pipeline::prepass::ViewPrepassTextures,
    ecs::system::lifetimeless::Read,
    pbr::ViewLightsUniformOffset,
    render::{
        extract_component::DynamicUniformIndex,
        render_graph::{RenderGraphContext, ViewNode},
        view::{ViewTarget, ViewUniformOffset},
    },
};

use super::op::SdOpUniformInstance;
use super::pipeline::RayMarchEnginePipeline;
use super::shape::SdShapeUniformInstance;
use super::{RayMarchCamera, RayMarchPrepass, SdOpStorage, SdShapeStorage, WORKGROUP_SIZE};

#[derive(Default)]
pub struct RayMarchEngineNode;

impl ViewNode for RayMarchEngineNode {
    type ViewQuery = (
        Read<ViewTarget>,
        Read<ExtractedCamera>,
        Read<RayMarchCamera>,
        Read<DynamicUniformIndex<RayMarchCamera>>,
        Read<ViewPrepassTextures>,
        Read<ViewUniformOffset>,
        Read<ViewLightsUniformOffset>,
        Read<RayMarchPrepass>,
    );

    fn run(
        &self,
        _graph: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        (
            _view_target,
            camera,
            _ray_march_settings,
            settings_index,
            view_prepass,
            view_uniform_offset,
            view_lights_uniform_offset,
            raymarch_prepass,
        ): QueryItem<Self::ViewQuery>,
        world: &World,
    ) -> Result<(), NodeRunError> {
        let ray_march_pipeline = world.resource::<RayMarchEnginePipeline>();
        let pipeline_cache = world.resource::<PipelineCache>();

        let Some(march_pipeline) =
            pipeline_cache.get_compute_pipeline(ray_march_pipeline.march_pipeline)
        else {
            return Ok(());
        };

        let Some(scale_pipeline) =
            pipeline_cache.get_compute_pipeline(ray_march_pipeline.scale_pipeline)
        else {
            return Ok(());
        };

        let settings_uniforms = world.resource::<ComponentUniforms<RayMarchCamera>>();
        let Some(settings_binding) = settings_uniforms.uniforms().binding() else {
            return Ok(());
        };

        let Some(depth_prepass) = view_prepass.depth_view() else {
            return Ok(());
        };

        let texture_bind_group = render_context.render_device().create_bind_group(
            "ray_march_texture_bind_group",
            &ray_march_pipeline.texture_layout,
            &BindGroupEntries::sequential((depth_prepass, settings_binding.clone())),
        );

        let view_uniforms = world.get_resource::<ViewUniforms>().unwrap();
        let Some(view_binding) = view_uniforms.uniforms.binding() else {
            return Ok(());
        };

        let light = world.get_resource::<LightMeta>().unwrap();
        let Some(light_binding) = light.view_gpu_lights.binding() else {
            return Ok(());
        };

        let clusterable_objects = world.get_resource::<GlobalClusterableObjectMeta>().unwrap();
        let Some(clusterables_objects_binding) =
            clusterable_objects.gpu_clusterable_objects.binding()
        else {
            return Ok(());
        };

        let common_bind_group = render_context.render_device().create_bind_group(
            "ray_march_view_bind_group",
            &ray_march_pipeline.common_layout,
            &BindGroupEntries::with_indices((
                (0, view_binding.clone()),
                (1, light_binding.clone()),
                (8, clusterables_objects_binding.clone()),
            )),
        );

        let sd_shape_ressource = world.resource::<SdShapeStorage>();
        let mut sd_shape_storage = BufferVec::<SdShapeUniformInstance>::new(BufferUsages::STORAGE);
        for &sd_shape in sd_shape_ressource.data.iter() {
            sd_shape_storage.push(SdShapeUniformInstance {
                shape: sd_shape.shape.uniform(),
                material: sd_shape.material.uniform(),
                modifier: sd_shape.modifier.uniform(),
                transform: sd_shape.transform.uniform(),
            });
        }

        let sd_op_ressource = world.resource::<SdOpStorage>();
        let mut sd_op_storage = BufferVec::<SdOpUniformInstance>::new(BufferUsages::STORAGE);
        for &sd_op in sd_op_ressource.data.iter() {
            sd_op_storage.push(sd_op.uniform());
        }

        let render_device = world.resource::<RenderDevice>();
        let render_queue = world.resource::<RenderQueue>();

        sd_shape_storage.write_buffer(render_device, render_queue);
        sd_op_storage.write_buffer(render_device, render_queue);

        let storage_bind_group = render_context.render_device().create_bind_group(
            "marcher_storage_bind_group",
            &ray_march_pipeline.storage_layout,
            &BindGroupEntries::sequential((
                sd_shape_storage.binding().unwrap(),
                sd_op_storage.binding().unwrap(),
            )),
        );

        let Some(depth_map) = world
            .resource::<RenderAssets<GpuImage>>()
            .get(raymarch_prepass.depth.id())
        else {
            return Ok(());
        };

        let Some(normal_map) = world
            .resource::<RenderAssets<GpuImage>>()
            .get(raymarch_prepass.normal.id())
        else {
            return Ok(());
        };

        let Some(material_map) = world
            .resource::<RenderAssets<GpuImage>>()
            .get(raymarch_prepass.material.id())
        else {
            return Ok(());
        };

        let Some(mask_map) = world
            .resource::<RenderAssets<GpuImage>>()
            .get(raymarch_prepass.mask.id())
        else {
            return Ok(());
        };

        let Some(scaled_depth_map) = world
            .resource::<RenderAssets<GpuImage>>()
            .get(raymarch_prepass.scaled_depth.id())
        else {
            return Ok(());
        };

        let Some(scaled_normal_map) = world
            .resource::<RenderAssets<GpuImage>>()
            .get(raymarch_prepass.scaled_normal.id())
        else {
            return Ok(());
        };

        let Some(scaled_material_map) = world
            .resource::<RenderAssets<GpuImage>>()
            .get(raymarch_prepass.scaled_material.id())
        else {
            return Ok(());
        };

        let Some(scaled_mask_map) = world
            .resource::<RenderAssets<GpuImage>>()
            .get(raymarch_prepass.scaled_mask.id())
        else {
            return Ok(());
        };

        let prepass_bind_group = render_context.render_device().create_bind_group(
            "marcher_prepass_bind_group",
            &ray_march_pipeline.prepass_layout,
            &BindGroupEntries::sequential((
                &depth_map.texture_view,
                &normal_map.texture_view,
                &material_map.texture_view,
                &mask_map.texture_view,
                &scaled_depth_map.texture_view,
                &scaled_normal_map.texture_view,
                &scaled_material_map.texture_view,
                &scaled_mask_map.texture_view,
            )),
        );

        let Some(viewport) = camera.physical_viewport_size else {
            return Ok(());
        };

        let mut compute_pass =
            render_context
                .command_encoder()
                .begin_compute_pass(&ComputePassDescriptor {
                    label: Some("ray_march_pass"),
                    timestamp_writes: None,
                });

        compute_pass.set_bind_group(
            0,
            &common_bind_group,
            &[
                view_uniform_offset.offset,
                view_lights_uniform_offset.offset,
            ],
        );
        compute_pass.set_bind_group(1, &texture_bind_group, &[settings_index.index()]);
        compute_pass.set_bind_group(2, &storage_bind_group, &[]);
        compute_pass.set_bind_group(3, &prepass_bind_group, &[]);

        compute_pass.set_pipeline(march_pipeline);
        compute_pass.dispatch_workgroups(
            viewport.x.div_ceil(WORKGROUP_SIZE),
            viewport.y.div_ceil(WORKGROUP_SIZE),
            1,
        );

        compute_pass.set_pipeline(scale_pipeline);
        compute_pass.dispatch_workgroups(
            viewport.x.div_ceil(WORKGROUP_SIZE),
            viewport.y.div_ceil(WORKGROUP_SIZE),
            1,
        );

        Ok(())
    }
}
