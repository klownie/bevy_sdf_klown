use bevy::ecs::query::QueryItem;
use bevy::pbr::{GlobalClusterableObjectMeta, LightMeta};
use bevy::prelude::*;
use bevy::render::extract_component::ComponentUniforms;
use bevy::render::render_graph::NodeRunError;
use bevy::render::render_resource::{
    BindGroupEntries, BufferUsages, BufferVec, Operations, PipelineCache,
    RenderPassColorAttachment, RenderPassDescriptor,
};
use bevy::render::renderer::{RenderContext, RenderDevice, RenderQueue};
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
use super::{RayMarchCamera, RayMarchEnginePipelineId, SdOpStorage, SdShapeStorage};

#[derive(Default)]
pub struct RayMarchEngineNode;

impl ViewNode for RayMarchEngineNode {
    type ViewQuery = (
        Read<ViewTarget>,
        Read<RayMarchCamera>,
        Read<DynamicUniformIndex<RayMarchCamera>>,
        Read<ViewPrepassTextures>,
        Read<ViewUniformOffset>,
        Read<ViewLightsUniformOffset>,
        Read<RayMarchEnginePipelineId>,
    );

    fn run(
        &self,
        _graph: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        (
            view_target,
            _ray_march_settings,
            settings_index,
            view_prepass,
            view_uniform_offset,
            view_lights_uniform_offset,
            pipeline_id,
        ): QueryItem<Self::ViewQuery>,
        world: &World,
    ) -> Result<(), NodeRunError> {
        let ray_march_pipeline = world.resource::<RayMarchEnginePipeline>();
        let pipeline_cache = world.resource::<PipelineCache>();

        let Some(pipeline) = pipeline_cache.get_render_pipeline(**pipeline_id) else {
            return Ok(());
        };

        let settings_uniforms = world.resource::<ComponentUniforms<RayMarchCamera>>();
        let Some(settings_binding) = settings_uniforms.uniforms().binding() else {
            return Ok(());
        };

        let post_process = view_target.post_process_write();

        let (Some(depth_prepass), Some(normal_prepass), Some(motion_prepass)) = (
            view_prepass.depth_view(),
            view_prepass.normal_view(),
            view_prepass.motion_vectors_view(),
        ) else {
            return Ok(());
        };

        let texture_bind_group = render_context.render_device().create_bind_group(
            "ray_march_texture_bind_group",
            &ray_march_pipeline.texture_layout,
            &BindGroupEntries::sequential((
                post_process.source,
                &ray_march_pipeline.sampler,
                depth_prepass,
                settings_binding.clone(),
            )),
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

        let mut render_pass = render_context.begin_tracked_render_pass(RenderPassDescriptor {
            label: Some("ray_march_pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: post_process.destination,
                resolve_target: None,
                ops: Operations::default(),
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        render_pass.set_render_pipeline(pipeline);
        render_pass.set_bind_group(
            0,
            &common_bind_group,
            &[
                view_uniform_offset.offset,
                view_lights_uniform_offset.offset,
            ],
        );
        render_pass.set_bind_group(1, &texture_bind_group, &[settings_index.index()]);
        render_pass.set_bind_group(2, &storage_bind_group, &[]);
        render_pass.draw(0..3, 0..1);

        Ok(())
    }
}
