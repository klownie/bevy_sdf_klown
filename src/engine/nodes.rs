use bevy::ecs::query::QueryItem;
use bevy::pbr::{GlobalClusterableObjectMeta, LightMeta};
use bevy::prelude::*;
use bevy::render::camera::ExtractedCamera;
use bevy::render::extract_component::ComponentUniforms;
use bevy::render::render_graph::NodeRunError;
use bevy::render::render_resource::{
    BindGroupEntries, BufferUsages, BufferVec, ComputePassDescriptor, PipelineCache,
};
use bevy::render::renderer::{RenderContext, RenderQueue};
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

use crate::engine::object::SdModUniform;

use super::object::SdObjectUniform;
use super::op::SdOpUniformInstance;
use super::pipeline::RayMarchEnginePipeline;
use super::prepass::RayMarchPrepass;
use super::{RayMarchCamera, SdObjectStorage, SdOpStorage, WORKGROUP_SIZE};

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
        let device = render_context.render_device();
        let queue = world.resource::<RenderQueue>();
        let pipeline_cache = world.resource::<PipelineCache>();
        let ray_march_pipeline = world.resource::<RayMarchEnginePipeline>();

        let (Some(march_pipeline), Some(scale_pipeline)) = (
            pipeline_cache.get_compute_pipeline(ray_march_pipeline.march_pipeline),
            pipeline_cache.get_compute_pipeline(ray_march_pipeline.scale_pipeline),
        ) else {
            return Ok(());
        };

        let settings_uniforms = world.resource::<ComponentUniforms<RayMarchCamera>>();
        let view_uniforms = world.resource::<ViewUniforms>();
        let light_meta = world.resource::<LightMeta>();
        let clusterables = world.resource::<GlobalClusterableObjectMeta>();

        let (
            Some(settings_binding),
            Some(view_binding),
            Some(light_binding),
            Some(cluster_binding),
        ) = (
            settings_uniforms.uniforms().binding(),
            view_uniforms.uniforms.binding(),
            light_meta.view_gpu_lights.binding(),
            clusterables.gpu_clusterable_objects.binding(),
        )
        else {
            return Ok(());
        };

        let texture_bind_group = device.create_bind_group(
            "ray_march_texture_bind_group",
            &ray_march_pipeline.texture_layout,
            &BindGroupEntries::sequential((
                // view_target.get_unsampled_color_attachment().view,
                view_prepass.depth_view().unwrap(),
                settings_binding.clone(),
            )),
        );

        let common_bind_group = device.create_bind_group(
            "ray_march_view_bind_group",
            &ray_march_pipeline.common_layout,
            &BindGroupEntries::with_indices((
                (0, view_binding.clone()),
                (1, light_binding.clone()),
                (8, cluster_binding.clone()),
            )),
        );

        let res_obj = &world.resource::<SdObjectStorage>().data;

        let mut sd_mod_buf = BufferVec::<SdModUniform>::new(BufferUsages::STORAGE);
        let mut sd_object_buf = BufferVec::<SdObjectUniform>::new(BufferUsages::STORAGE);
        sd_object_buf.reserve(res_obj.len(), device);

        let mut current_mod_index = 0;

        for obj in res_obj.iter() {
            // Push modifiers and count them
            let start_index = current_mod_index;
            for &modifier in obj.modifiers.modifiers.iter().rev() {
                sd_mod_buf.push(modifier.uniform());
                current_mod_index += 1;
            }

            sd_object_buf.push(SdObjectUniform {
                shape: obj.shape.clone().uniform(),
                material: obj.material.uniform(),
                modifiers: obj.modifiers.clone().uniform(start_index),
                transform: obj.transform.uniform(),
            });
        }

        sd_mod_buf.reserve(current_mod_index, device);

        let mut sd_op_buf = BufferVec::<SdOpUniformInstance>::new(BufferUsages::STORAGE);
        let res_op = &world.resource::<SdOpStorage>().data;
        sd_op_buf.reserve(res_op.len(), device);
        for &op in res_op.iter() {
            sd_op_buf.push(op.uniform());
        }

        sd_mod_buf
            .is_empty()
            .then(|| sd_mod_buf.push(SdModUniform::default()));

        sd_mod_buf.write_buffer(device, queue);
        sd_object_buf.write_buffer(device, queue);
        sd_op_buf.write_buffer(device, queue);

        let storage_bind_group = device.create_bind_group(
            "marcher_storage_bind_group",
            &ray_march_pipeline.storage_layout,
            &BindGroupEntries::sequential((
                sd_mod_buf.binding().unwrap(),
                sd_object_buf.binding().unwrap(),
                sd_op_buf.binding().unwrap(),
            )),
        );

        let prepass_bind_group = device.create_bind_group(
            "marcher_prepass_bind_group",
            &ray_march_pipeline.prepass_layout,
            &BindGroupEntries::sequential((
                &raymarch_prepass.depth,
                &raymarch_prepass.normal,
                &raymarch_prepass.material,
                &raymarch_prepass.mask,
                &raymarch_prepass.scaled_depth,
                &raymarch_prepass.scaled_normal,
                &raymarch_prepass.scaled_material,
                &raymarch_prepass.scaled_mask,
            )),
        );

        let Some(viewport) = camera.physical_viewport_size else {
            return Ok(());
        };

        let mut pass =
            render_context
                .command_encoder()
                .begin_compute_pass(&ComputePassDescriptor {
                    label: Some("ray_march_pass"),
                    timestamp_writes: None,
                });

        pass.set_bind_group(
            0,
            &common_bind_group,
            &[
                view_uniform_offset.offset,
                view_lights_uniform_offset.offset,
            ],
        );
        pass.set_bind_group(1, &texture_bind_group, &[settings_index.index()]);
        pass.set_bind_group(2, &storage_bind_group, &[]);
        pass.set_bind_group(3, &prepass_bind_group, &[]);

        pass.set_pipeline(march_pipeline);
        pass.dispatch_workgroups(
            viewport.x.div_ceil(WORKGROUP_SIZE),
            viewport.y.div_ceil(WORKGROUP_SIZE),
            1,
        );

        pass.set_pipeline(scale_pipeline);
        pass.dispatch_workgroups(
            viewport.x.div_ceil(WORKGROUP_SIZE),
            viewport.y.div_ceil(WORKGROUP_SIZE),
            1,
        );

        Ok(())
    }
}
